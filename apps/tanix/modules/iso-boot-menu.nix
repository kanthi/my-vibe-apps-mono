# Boot menu of both Tanix ISOs (desktop and server): classic white text on a
# black screen, with the selected entry shown as a white bar with black text.
{ pkgs, ... }:

let
  bootMenuTheme = pkgs.runCommand "tanix-grub-theme" { nativeBuildInputs = [ pkgs.resvg ]; } ''
    mkdir -p $out

    square() {
      echo "<svg xmlns='http://www.w3.org/2000/svg' width='16' height='16'><rect width='16' height='16' fill='$1'/></svg>" > tile.svg
      resvg tile.svg $2
    }
    # GRUB needs a background image even when it is a single colour.
    square '#000000' $out/background.png
    square '#ffffff' $out/select_c.png

    # "Unifont Regular 16" is the font the ISO already loads (unicode.pf2).
    cat > $out/theme.txt <<'EOF'
    title-text: "Tanix"
    title-font: "Unifont Regular 16"
    title-color: "#ffffff"
    desktop-image: "background.png"
    desktop-color: "#000000"
    message-font: "Unifont Regular 16"
    message-color: "#ffffff"
    terminal-font: "Unifont Regular 16"

    + boot_menu {
      left = 10%
      width = 80%
      top = 15%
      height = 60%
      item_font = "Unifont Regular 16"
      item_color = "#ffffff"
      selected_item_font = "Unifont Regular 16"
      selected_item_color = "#000000"
      selected_item_pixmap_style = "select_*.png"
      item_height = 22
      item_padding = 2
      item_spacing = 2
      icon_width = 0
      icon_height = 0
      item_icon_space = 0
      scrollbar = false
    }

    + progress_bar {
      id = "__timeout__"
      left = 10%
      width = 80%
      top = 82%
      height = 22
      show_text = true
      font = "Unifont Regular 16"
      text_color = "#ffffff"
      text = "@TIMEOUT_NOTIFICATION_LONG@"
      border_color = "#ffffff"
      bg_color = "#000000"
      fg_color = "#444444"
    }
    EOF
  '';
in
{
  # Boot menu theme (UEFI). The BIOS menu, used only on x86, gets the same
  # plain black background.
  isoImage.grubTheme = bootMenuTheme;
  isoImage.splashImage = "${bootMenuTheme}/background.png";
}
