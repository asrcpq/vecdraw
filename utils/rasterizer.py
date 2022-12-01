import json
import sys
from PIL import Image, ImageDraw
j = json.load(open(sys.argv[1]))
r = 4096
im = Image.new("RGBA", [r, r], "#00000000")
draw = ImageDraw.Draw(im)
for [[v1, v2], _] in j["dcs"]:
	p1 = j["vs"][str(v1)]["tex"]
	p2 = j["vs"][str(v2)]["tex"]
	x1 = p1[0] * r;
	y1 = p1[1] * r;
	x2 = p2[0] * r;
	y2 = p2[1] * r;
	draw.line((x1, y1, x2, y2), fill = "#ff00ffff", width = 2)
im = im.resize((r // 2, r // 2), resample = Image.Resampling.LANCZOS)
im.save(sys.argv[2], "PNG")

