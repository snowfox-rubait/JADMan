(function() {
    const DEBUG = true;
    if (window.__JADMAN_SIPHON_LOADED__) return;
    window.__JADMAN_SIPHON_LOADED__ = true;

    function log(...args) {
        if (DEBUG) console.log('[JADMan Siphon]', ...args);
    }

    function dispatch(url, type, data, mime) {
        // Detect if this is a manifest or a segment
        let priority = 'NORMAL';
        const lowerUrl = url.toLowerCase();
        
        if (lowerUrl.includes('.m3u8') || lowerUrl.includes('.mpd') || mime === 'application/x-mpegURL' || mime === 'application/dash+xml') {
            priority = 'MANIFEST';
        } else if (lowerUrl.includes('.ts') || lowerUrl.includes('.m4s') || lowerUrl.includes('.m4a') || lowerUrl.includes('.m4v')) {
            priority = 'SEGMENT';
        }

        window.postMessage({
            source: 'JADMAN_SIPHON',
            url: url,
            type: type, // FETCH or XHR
            priority: priority,
            data: data, 
            mime: mime
        }, '*');
    }

    // STEALTH PROXY UTILITY FOR HOOKS
    const proxyOriginalMap = new WeakMap();
    const originalToString = Function.prototype.toString;

    Function.prototype.toString = new Proxy(originalToString, {
        apply(target, thisArg, argArray) {
            if (proxyOriginalMap.has(thisArg)) {
                const original = proxyOriginalMap.get(thisArg);
                return originalToString.call(original);
            }
            return originalToString.apply(thisArg, argArray);
        }
    });

    function createStealthProxy(original, hookImpl) {
        const handler = {
            apply(target, thisArg, argArray) {
                return hookImpl.apply(thisArg, argArray);
            },
            construct(target, argArray, newTarget) {
                return Reflect.construct(hookImpl, argArray, newTarget);
            },
            get(target, prop, receiver) {
                if (prop === 'toString') {
                    return function toString() {
                        return originalToString.call(original);
                    };
                }
                const val = Reflect.get(target, prop, receiver);
                return typeof val === 'function' ? val.bind(target) : val;
            }
        };
        const proxy = new Proxy(original, handler);
        proxyOriginalMap.set(proxy, original);
        return proxy;
    }

    // HIJACK FETCH
    const originalFetch = window.fetch;
    window.fetch = createStealthProxy(originalFetch, async function(...args) {
        const response = await originalFetch.apply(this, args);
        const url = response.url;
        const lowerUrl = url.toLowerCase();

        // Check if it's something we care about (Manifests or Large Blobs)
        const isManifest = lowerUrl.includes('.m3u8') || lowerUrl.includes('.mpd');
        const isSegment = lowerUrl.includes('.ts') || lowerUrl.includes('.m4s');

        if (isManifest || isSegment) {
            const clone = response.clone();
            clone.arrayBuffer().then(buffer => {
                dispatch(url, 'FETCH', buffer, clone.headers.get('content-type'));
            }).catch(() => {});
        }

        return response;
    });

    // HIJACK XHR
    const originalOpen = XMLHttpRequest.prototype.open;
    const originalSend = XMLHttpRequest.prototype.send;

    XMLHttpRequest.prototype.open = createStealthProxy(originalOpen, function(method, url) {
        this._url = url;
        return originalOpen.apply(this, arguments);
    });

    XMLHttpRequest.prototype.send = createStealthProxy(originalSend, function() {
        this.addEventListener('load', function() {
            try {
                const url = this.responseURL || this._url;
                const lowerUrl = url.toLowerCase();
                const contentType = this.getResponseHeader('content-type') || "";
                
                const isManifest = lowerUrl.includes('.m3u8') || lowerUrl.includes('.mpd') || contentType.includes('mpegURL') || contentType.includes('dash+xml');
                const isSegment = lowerUrl.includes('.ts') || lowerUrl.includes('.m4s');

                if (isManifest || isSegment) {
                    let data = (this.responseType === 'arraybuffer' || this.responseType === 'blob') ? this.response : this.responseText;
                    if (data) {
                        dispatch(url, 'XHR', data, contentType);
                    }
                }
            } catch (e) {}
        });
        return originalSend.apply(this, arguments);
    });

    // HIJACK MEDIASOURCE & SOURCEBUFFER (Decrypted Stream Interception)
    try {
        if (window.MediaSource) {
            const originalAddSourceBuffer = MediaSource.prototype.addSourceBuffer;
            MediaSource.prototype.addSourceBuffer = createStealthProxy(originalAddSourceBuffer, function(mimeType) {
                const sourceBuffer = originalAddSourceBuffer.call(this, mimeType);
                sourceBuffer._mimeType = mimeType;
                log('Hooked addSourceBuffer with mimeType:', mimeType);
                return sourceBuffer;
            });
        }

        if (window.SourceBuffer) {
            const originalAppendBuffer = SourceBuffer.prototype.appendBuffer;
            SourceBuffer.prototype.appendBuffer = createStealthProxy(originalAppendBuffer, function(data) {
                try {
                    const mime = this._mimeType || 'video/mp4';
                    let buffer;
                    if (data instanceof ArrayBuffer) {
                        buffer = data;
                    } else if (data && data.buffer instanceof ArrayBuffer) {
                        buffer = data.buffer;
                    }

                    if (buffer && buffer.byteLength > 0) {
                        dispatch(window.location.href, 'APPEND_BUFFER', buffer.slice(0), mime);
                    }
                } catch (e) {
                    log('Error in appendBuffer hook:', e);
                }
                return originalAppendBuffer.apply(this, arguments);
            });
        }
    } catch (e) {
        log('Failed to install MediaSource hooks:', e);
    }

    log('Deep Capture Hook (v2: Stream Aware) Active.');
})();
