use axohtml::{elements::div, html, text};
use crate::database::{Capture, get_captures};

fn generate_info(capture: &Capture) -> Box<div<String>> {
    let cap_id_str = capture.id.to_string();

    let mut classes = "p-2 text-white bg-red-600 opacity-90";
    if capture.success {
        classes = "p-2 text-white bg-green-600 opacity-90";
    }
    let mut a11y_capture_info = "capture failed ";
    if capture.success {
        a11y_capture_info = "capture succeeded ";
    }

    // tbh this macro is a big hack and I don't like how it interacts with VS Code. The fact
    // I can't collapse DOM nodes is not dyslexia friendly. Ah well.
    html!(
        <div role="group" class=classes>
            <p class="text-sm block" tabindex="0">
                <span class="sr-only">{text!(a11y_capture_info)}</span>
                {text!(&capture.filename)}
            </p>

            <div class="hide-first-when-hovered text-sm">
                <div class="hide-first-when-hovered__first">
                    <p tabindex="0">
                        <span class="sr-only">"created at"</span>
                        <i tabindex="-1" class="fa-regular fa-clock"></i>
                        {text!(' ')}{text!(&capture.created_at)}
                    </p>
                </div>

                <div class="hide-first-when-hovered__second">    
                    <div class="flex">           
                        <div class="flex-col">
                            <form
                                data-action="copyUrl" method="post" autocomplete="off"
                            >
                                <input type="hidden" name="capture_id" value=&cap_id_str />
                                <button class="cursor-default" type="submit" aria-label="Copy URL">
                                    <i class="fa-regular fa-copy"></i>
                                </button>
                            </form>
                        </div>

                        <div class="flex-col ml-2">
                            <form
                                data-action="openUrl" method="post" autocomplete="off"
                            >
                                <input type="hidden" name="capture_id" value=&cap_id_str />
                                <button class="cursor-default" type="submit" aria-label="Open URL">
                                    <i class="fa-regular fa-external-link"></i>
                                </button>
                            </form>
                        </div>

                        <div class="flex-col ml-2">
                            <form
                                data-action="showInFolder" method="post" autocomplete="off"
                            >
                                <input type="hidden" name="capture_id" value=&cap_id_str />
                                <button class="cursor-default" type="submit" aria-label="Show in Folder">
                                    <i class="fa-regular fa-folder"></i>
                                </button>
                            </form>
                        </div>

                        <div class="flex-col ml-2">
                            <form
                                data-action="openFile" method="post" autocomplete="off"
                            >
                                <input type="hidden" name="capture_id" value=&cap_id_str />
                                <button class="cursor-default" type="submit" aria-label="Open File">
                                    <i class="fa-regular fa-file"></i>
                                </button>
                            </form>
                        </div>

                        <div class="flex-col ml-2">
                            <form
                                data-action="deleteCapture" method="post"
                                autocomplete="off" aria-relevant="all" aria-live="assertive"
                            >
                                <input type="hidden" name="capture_id" value=&cap_id_str />
                                <button class="cursor-default" type="submit" aria-label="Delete Capture">
                                    <i class="fa-regular fa-trash"></i>
                                </button>
                            </form>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    )
}

pub fn generate_dom_node(capture: Capture) -> Box<div<String>> {
    let blowaway_var: String;
    let fp = match &capture.file_path {
        Some(fp) => fp,
        None => {
            blowaway_var = "".to_string();
            &blowaway_var
        },
    };

    html!(
        <div data-capture-root="1" class="flex-col m-2 shadow-md">
            <div class="block w-48 h-24 relative rounded-lg overflow-hidden">
                <div class="absolute w-full z-10 bottom-0">
                    {generate_info(&capture)}
                </div>
                <img
                    class="object-cover w-full h-full rounded-lg absolute"
                    src="magiccap-internal://frontend-dist/placeholder.png"
                    alt=""
                    loading="lazy"
                    data-filesystem-path=fp
                />
            </div>
        </div>
    )
}

pub fn captures_html() -> Vec<u8> {
    html!(
        <div class="flex flex-wrap justify-center">
            {get_captures().into_iter().map(|c| generate_dom_node(c))}
        </div>
    ).to_string().into_bytes()
}
