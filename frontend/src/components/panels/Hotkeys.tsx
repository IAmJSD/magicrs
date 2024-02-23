import Container from "../atoms/Container";
import Divider from "../atoms/Divider";
import Header from "../atoms/Header";
import Hotkey from "../atoms/config/Hotkey";

export default function Hotkeys() {
    return <Container>
        <Header
            title="Hotkeys"
            subtitle="Configures the hotkeys used to control MagicCap:"
        />

        <Hotkey
            dbKey="region_hotkey"
            label="Region Capture"
            description="Defines the hotkey to open the region selector and capture a image:"
        />

        <Divider />

        <Hotkey
            dbKey="fullscreen_hotkey"
            label="Fullscreen Capture"
            description="Defines the hotkey to capture the entire screen:"
        />

        <Divider />

        <Hotkey
            dbKey="gif_hotkey"
            label="GIF Capture"
            description="Defines the hotkey to capture GIF's:"
        />

        <Divider />

        <Hotkey
            dbKey="video_hotkey"
            label="Video Capture"
            description="Defines the hotkey to capture videos:"
        />

        <Divider />

        <Hotkey
            dbKey="clipboard_hotkey"
            label="Clipboard Capture"
            description="Defines the hotkey to capture the clipboard:"
        />
    </Container>;
}
