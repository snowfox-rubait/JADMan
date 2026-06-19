import sys
import os
import argparse
import time
import uuid
from camoufox.sync_api import Camoufox

def main():
    parser = argparse.ArgumentParser(description="Download a file using Camoufox (Playwright)")
    parser.add_argument("--url", required=True, help="URL to download")
    parser.add_argument("--folder", required=True, help="Destination folder")
    parser.add_argument("--user-agent", help="User Agent to spoof")
    parser.add_argument("--cookies-file", help="Path to Netscape cookies file")
    args = parser.parse_args()

    # Ensure dest folder exists
    os.makedirs(args.folder, exist_ok=True)

    print(f"[Camoufox Downloader] Target URL: {args.url}")
    print(f"[Camoufox Downloader] Output folder: {args.folder}")
    
    # Run headless Camoufox
    with Camoufox(headless=True) as browser:
        context = browser.new_context(user_agent=args.user_agent) if args.user_agent else browser.new_context()

        # Load Netscape cookies if provided
        if args.cookies_file and os.path.exists(args.cookies_file):
            print(f"[Camoufox Downloader] Loading cookies from {args.cookies_file}")
            playwright_cookies = []
            try:
                with open(args.cookies_file, "r") as f:
                    for line in f:
                        if line.startswith("#") or not line.strip():
                            continue
                        parts = line.strip().split("\t")
                        if len(parts) >= 7:
                            domain = parts[0]
                            path = parts[2]
                            secure = parts[3].upper() == "TRUE"
                            expires = int(parts[4])
                            name = parts[5]
                            value = parts[6]
                            
                            cookie = {
                                "name": name,
                                "value": value,
                                "domain": domain,
                                "path": path,
                                "secure": secure,
                            }
                            if expires > 0:
                                cookie["expires"] = expires
                            playwright_cookies.append(cookie)
                if playwright_cookies:
                    context.add_cookies(playwright_cookies)
            except Exception as e:
                print(f"[Camoufox Downloader] Error parsing Netscape cookies: {e}")

        page = context.new_page()

        # Setup route interception to force content-disposition header to attachment
        target_url = args.url
        download_obj = None

        def handle_route(route):
            try:
                req_url = route.request.url.split('?')[0]
                tgt_url = target_url.split('?')[0]
                if req_url == tgt_url:
                    print("[Camoufox Downloader] Intercepting target request to force download...")
                    response = page.request.fetch(route.request)
                    headers = response.headers
                    
                    filename = target_url.split('/').pop().split('?')[0] or "download"
                    headers["content-disposition"] = f"attachment; filename={filename}"
                    
                    route.fulfill(
                        response=response,
                        headers=headers
                    )
                else:
                    route.continue_()
            except Exception as e:
                print(f"[Camoufox Downloader] Error in route interception: {e}")
                route.continue_()

        page.route("**/*", handle_route)

        print("[Camoufox Downloader] Triggering request...")
        try:
            with page.expect_download(timeout=45000) as download_info:
                try:
                    page.goto(target_url, wait_until="commit")
                except Exception as e:
                    pass
            download_obj = download_info.value
        except Exception as e:
            print(f"[Camoufox Downloader] No download event triggered: {e}")

        if download_obj:
            filename = download_obj.suggested_filename
            
            # 1. Download to a unique temporary filename
            temp_filename = f"cf_temp_{uuid.uuid4()}.tmp"
            temp_path = os.path.join(args.folder, temp_filename)
            
            print(f"[Camoufox Downloader] Download started. Saving to temp path: {temp_path}")
            download_obj.save_as(temp_path)
            
            # 2. Download is complete here. Now resolve unique path in destination folder
            dest_path = os.path.join(args.folder, filename)
            if os.path.exists(dest_path):
                stem, ext = os.path.splitext(filename)
                counter = 1
                while True:
                    new_filename = f"{stem} ({counter}){ext}"
                    new_path = os.path.join(args.folder, new_filename)
                    if not os.path.exists(new_path):
                        dest_path = new_path
                        break
                    counter += 1

            # 3. Rename temp file to unique final path
            try:
                os.rename(temp_path, dest_path)
                print(f"[Camoufox Downloader] Download completed successfully: {dest_path}")
                sys.exit(0)
            except Exception as e:
                print(f"[Camoufox Downloader] Error moving temp file to destination: {e}")
                sys.exit(1)
        else:
            print("[Camoufox Downloader] Error: No download captured.")
            sys.exit(1)

if __name__ == "__main__":
    main()
