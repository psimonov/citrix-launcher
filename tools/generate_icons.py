from pathlib import Path

from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "assets" / "icons"
SIZES = (16, 24, 32, 48, 64, 128, 256, 512, 1024)
ARTWORK_CROP = (44, 44, 980, 980)


def draw_master() -> Image.Image:
    image = Image.new("RGBA", (1024, 1024), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)

    # Papirus-inspired layered silhouette: dark shadow below, lighter surface above.
    draw.rounded_rectangle((88, 126, 936, 770), 104, fill="#143B78")
    draw.rounded_rectangle((88, 96, 936, 740), 104, fill="#3478E5")
    draw.rounded_rectangle((112, 112, 912, 716), 84, outline=(255, 255, 255, 48), width=10)

    # One large screen shape, with no title-bar controls or other small decoration.
    draw.rounded_rectangle((176, 190, 848, 636), 52, fill="#EAF6FF")
    draw.rounded_rectangle((196, 210, 828, 616), 38, fill="#D7EDFF")

    # Remote hand-off: a single bold arrow that remains recognizable at 16 px.
    draw.rounded_rectangle((280, 354, 632, 478), 42, fill="#08B8D4")
    draw.polygon(((590, 278), (804, 416), (590, 554)), fill="#08B8D4")
    draw.line(((300, 372), (604, 372)), fill=(255, 255, 255, 52), width=10)

    # Simple monitor stand and base, both aligned to the pixel grid.
    draw.polygon(((432, 740), (592, 740), (624, 860), (400, 860)), fill="#143B78")
    draw.rounded_rectangle((304, 842, 720, 938), 42, fill="#143B78")
    draw.rounded_rectangle((326, 842, 698, 916), 32, fill="#2563C7")
    return image.crop(ARTWORK_CROP).resize((1024, 1024), Image.Resampling.LANCZOS)


def write_svg() -> None:
    svg = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
<g transform="translate(-48.137 -48.137) scale(1.094017)">
  <rect x="88" y="126" width="848" height="644" rx="104" fill="#143b78"/>
  <rect x="88" y="96" width="848" height="644" rx="104" fill="#3478e5"/>
  <rect x="112" y="112" width="800" height="604" rx="84" fill="none" stroke="#fff" stroke-opacity=".19" stroke-width="10"/>
  <rect x="176" y="190" width="672" height="446" rx="52" fill="#eaf6ff"/>
  <rect x="196" y="210" width="632" height="406" rx="38" fill="#d7edff"/>
  <rect x="280" y="354" width="352" height="124" rx="42" fill="#08b8d4"/>
  <path d="M590 278 804 416 590 554Z" fill="#08b8d4"/>
  <path d="M300 372H604" fill="none" stroke="#fff" stroke-opacity=".2" stroke-width="10"/>
  <path d="M432 740H592L624 860H400Z" fill="#143b78"/>
  <rect x="304" y="842" width="416" height="96" rx="42" fill="#143b78"/>
  <rect x="326" y="842" width="372" height="74" rx="32" fill="#2563c7"/>
</g>
</svg>
"""
    (OUT / "icon-master.svg").write_text(svg, encoding="utf-8")


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    master = draw_master()
    for size in SIZES:
        icon = master.resize((size, size), Image.Resampling.LANCZOS)
        icon.save(OUT / f"icon-{size}.png", optimize=True)
    master.save(OUT / "icon-master.png", optimize=True)
    master.save(
        OUT / "citrix-vdi-launcher.ico",
        format="ICO",
        sizes=[(size, size) for size in (16, 24, 32, 48, 64, 128, 256)],
    )
    master.save(OUT / "citrix-vdi-launcher.icns", format="ICNS")
    write_svg()


if __name__ == "__main__":
    main()
