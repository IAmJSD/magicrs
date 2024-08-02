import ColorInput from "../atoms/config/ColorInput";
import Container from "../atoms/Container";
import Divider from "../atoms/Divider";
import Header from "../atoms/Header";
import OpenSourceCredits from "../molecules/OpenSourceCredits";

export default function General() {
    return <Container>
        <Header
            title="General"
            subtitle="Configures general settings for MagicCap:"
        />

        <ColorInput
            dbKey="default_editor_color"
            label="Default Editor Color"
            description="Defines the default color of the editor. This color is used when rendering shapes to the screen."
            defaultValue="#FF0000"
        />

        <Divider />

        <OpenSourceCredits />
    </Container>;
}
