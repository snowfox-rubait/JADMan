"""
JADMan Frida CDM Key Extractor
==============================
Attaches to a running Chromium/Chrome process, hooks the Widevine CDM library
(libwidevinecdm.so), and dumps Widevine L3 decryption keys as the browser
decrypts media. Keys are printed as:
    KEY:<key_id_hex>:<key_hex>
and also written to /tmp/jadm_cdm_keys.txt

Usage:
    python3 scripts/cdm_extractor.py [--pid <pid>] [--output /path/to/keys.txt]

If --pid is not given, auto-detects the first Chromium process.
"""

import frida
import sys
import os
import argparse
import json
import time

# Frida JS script that hooks inside the CDM process
FRIDA_HOOK_SCRIPT = r"""
'use strict';

// ──────────────────────────────────────────────────────────────────────────────
// Widevine L3 CDM key dump hook
// Targets: OEMCrypto_DeriveKeysFromSessionKey / ::DecryptCENC / UpdateSession
// The most reliable cross-platform hook is on the GetKeyStatus / UpdateSession
// path where the CDM writes the decrypted content key into a usable buffer.
// ──────────────────────────────────────────────────────────────────────────────

function toHex(arrayBuffer) {
    return Array.from(new Uint8Array(arrayBuffer))
        .map(b => ('00' + b.toString(16)).slice(-2))
        .join('');
}

// Chromium's CDM interface exposes cdm::ContentDecryptionModule via the
// cdm::Host_12 vtable. The key landing zone is cdm::Host::OnSessionKeysChange
// which receives key_information[] with the raw key_id and key.
//
// Strategy: hook dlopen/dlsym to detect when libwidevinecdm.so is loaded,
// then hook the exported Widevine functions.

const dumpedKeys = new Set();

function hookWidevine(baseAddr, moduleName) {
    send({ type: 'status', msg: 'Hooked module: ' + moduleName + ' @ ' + baseAddr });

    // Enumerate exports looking for Widevine's CreateCdmInstance
    const exports = Module.enumerateExports(moduleName);
    for (const exp of exports) {
        if (exp.name.includes('CreateCdmInstance') || exp.name.includes('INITIALIZE_CDM')) {
            send({ type: 'status', msg: 'Found CDM entrypoint: ' + exp.name });
        }
    }

    // Hook OEMCrypto_DeriveKeysFromSessionKey — always present in L3 software CDM
    // Its signature: OEMCryptoResult OEMCrypto_DeriveKeysFromSessionKey(
    //     OEMCrypto_SESSION session,
    //     const uint8_t* enc_session_key, size_t enc_session_key_length,
    //     const uint8_t* mac_key_context, size_t mac_key_context_length,
    //     uint8_t* enc_mac_key, uint8_t* sign_mac_key,
    //     const OEMCrypto_KeyObject* key_array, size_t num_keys);
    try {
        const fn = Module.findExportByName(moduleName, 'OEMCrypto_DeriveKeysFromSessionKey');
        if (fn) {
            Interceptor.attach(fn, {
                onLeave: function(retval) {
                    if (retval.toInt32() === 0) {
                        send({ type: 'status', msg: '[CDM] DeriveKeysFromSessionKey completed OK (keys now in memory)' });
                    }
                }
            });
            send({ type: 'status', msg: 'Attached to OEMCrypto_DeriveKeysFromSessionKey' });
        }
    } catch(e) {}

    // Hook OEMCrypto_LoadKeys — the function that actually installs content keys
    // OEMCryptoResult OEMCrypto_LoadKeys(
    //     OEMCrypto_SESSION session,
    //     const uint8_t* message, size_t message_length,
    //     const uint8_t* signature, size_t signature_length,
    //     const uint8_t* enc_mac_key, size_t enc_mac_key_length,
    //     const uint8_t* sign_mac_key, size_t sign_mac_key_length,
    //     const OEMCrypto_KeyObject* key_array, size_t num_keys,
    //     const uint8_t* pst, size_t pst_length,
    //     const uint8_t** srm_requirement, OEMCrypto_Usage_Entry_e *usage_entry);
    try {
        const fn = Module.findExportByName(moduleName, 'OEMCrypto_LoadKeys');
        if (fn) {
            Interceptor.attach(fn, {
                onEnter: function(args) {
                    // args[8] = key_array ptr, args[9] = num_keys
                    try {
                        const numKeys = args[9].toInt32();
                        if (numKeys <= 0 || numKeys > 32) return;

                        // OEMCrypto_KeyObject layout (64-bit):
                        //   uint8_t* key_id;           +0
                        //   size_t   key_id_length;    +8
                        //   uint8_t* key_data_iv;      +16
                        //   uint8_t* key_data;         +24
                        //   size_t   key_data_length;  +32
                        //   uint8_t* key_control;      +40
                        //   uint32_t cipher_mode;      +48
                        const KEY_OBJ_SIZE = 56;
                        const keyArrayPtr = args[8];

                        for (let i = 0; i < numKeys; i++) {
                            const obj = keyArrayPtr.add(i * KEY_OBJ_SIZE);
                            const keyIdPtr    = obj.readPointer();
                            const keyIdLen    = obj.add(8).readUSize();
                            const keyDataPtr  = obj.add(24).readPointer();
                            const keyDataLen  = obj.add(32).readUSize();

                            if (keyIdLen > 0 && keyIdLen <= 64 && keyDataLen > 0 && keyDataLen <= 64) {
                                const keyId  = toHex(keyIdPtr.readByteArray(keyIdLen));
                                const keyVal = toHex(keyDataPtr.readByteArray(keyDataLen));
                                const pair = keyId + ':' + keyVal;
                                if (!dumpedKeys.has(pair)) {
                                    dumpedKeys.add(pair);
                                    send({ type: 'key', key_id: keyId, key: keyVal });
                                }
                            }
                        }
                    } catch(e) {
                        // Pointer read failed — ignore silently
                    }
                }
            });
            send({ type: 'status', msg: 'Attached to OEMCrypto_LoadKeys — waiting for keys...' });
        } else {
            send({ type: 'status', msg: 'OEMCrypto_LoadKeys not found — may be inlined or obfuscated' });
        }
    } catch(e) {
        send({ type: 'status', msg: 'Hook error: ' + e.message });
    }
}

// Watch for libwidevinecdm.so being loaded (if not already loaded)
function watchForCDM() {
    const already = Process.enumerateModules().filter(m => m.name.includes('widevinecdm'));
    if (already.length > 0) {
        for (const m of already) hookWidevine(m.base, m.name);
        return;
    }

    // Not loaded yet — hook dlopen to catch it
    const dlopen = Module.findExportByName(null, 'dlopen');
    if (dlopen) {
        Interceptor.attach(dlopen, {
            onEnter: function(args) {
                this.path = args[0].readUtf8String();
            },
            onLeave: function(retval) {
                if (this.path && this.path.includes('widevinecdm')) {
                    const mod = Process.findModuleByName(this.path.split('/').pop());
                    if (mod) hookWidevine(mod.base, mod.name);
                }
            }
        });
        send({ type: 'status', msg: 'Watching dlopen for libwidevinecdm.so...' });
    }
}

watchForCDM();
"""


def main():
    parser = argparse.ArgumentParser(description='JADMan CDM Key Extractor')
    parser.add_argument('--pid', type=int, default=None, help='Chrome/Chromium PID to attach to')
    parser.add_argument('--output', default='/tmp/jadm_cdm_keys.txt', help='Key output file path')
    parser.add_argument('--timeout', type=int, default=120, help='How long to listen for keys (seconds)')
    args = parser.parse_args()

    output_path = args.output
    keys_collected = []

    # Auto-detect browser PID
    target_pid = args.pid
    if not target_pid:
        try:
            device = frida.get_local_device()
            for proc in device.enumerate_processes():
                name = proc.name.lower()
                if 'chrome' in name or 'chromium' in name:
                    target_pid = proc.pid
                    print(f"[JADMan CDM] Auto-detected browser: {proc.name} (PID {proc.pid})")
                    break
        except Exception as e:
            print(f"[JADMan CDM] ERROR: Could not enumerate processes: {e}", file=sys.stderr)
            sys.exit(1)

    if not target_pid:
        print("[JADMan CDM] ERROR: No Chrome/Chromium process found. Is the browser running?", file=sys.stderr)
        sys.exit(1)

    def on_message(message, data):
        if message['type'] == 'send':
            payload = message['payload']
            if payload.get('type') == 'key':
                key_id = payload['key_id']
                key    = payload['key']
                pair   = f"{key_id}:{key}"
                if pair not in [k['pair'] for k in keys_collected]:
                    keys_collected.append({'pair': pair, 'key_id': key_id, 'key': key})
                    print(f"KEY:{pair}")
                    sys.stdout.flush()
                    # Write to output file immediately
                    with open(output_path, 'a') as f:
                        f.write(f"{pair}\n")
            elif payload.get('type') == 'status':
                print(f"[CDM] {payload['msg']}", file=sys.stderr)
        elif message['type'] == 'error':
            print(f"[CDM ERROR] {message['stack']}", file=sys.stderr)

    print(f"[JADMan CDM] Attaching to PID {target_pid}...")
    try:
        session = frida.attach(target_pid)
        script  = session.create_script(FRIDA_HOOK_SCRIPT)
        script.on('message', on_message)
        script.load()
        print(f"[JADMan CDM] Hook active. Listening for Widevine keys (timeout: {args.timeout}s)...")
        print(f"[JADMan CDM] Key output: {output_path}")
        print(f"[JADMan CDM] Play a DRM-protected video now to trigger key extraction.")
    except frida.ProcessNotFoundError:
        print(f"[JADMan CDM] ERROR: PID {target_pid} not found or access denied.", file=sys.stderr)
        sys.exit(1)
    except frida.NotSupportedError as e:
        print(f"[JADMan CDM] ERROR: Frida not supported: {e}", file=sys.stderr)
        sys.exit(1)

    try:
        time.sleep(args.timeout)
    except KeyboardInterrupt:
        pass

    print(f"\n[JADMan CDM] Session ended. Collected {len(keys_collected)} key(s).")
    if keys_collected:
        print(f"[JADMan CDM] Keys written to: {output_path}")
        # Also write JSON summary
        summary_path = output_path.replace('.txt', '_summary.json')
        with open(summary_path, 'w') as f:
            json.dump({'keys': keys_collected, 'count': len(keys_collected)}, f, indent=2)
        print(f"[JADMan CDM] JSON summary: {summary_path}")

    session.detach()

if __name__ == '__main__':
    main()
