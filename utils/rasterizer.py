import json
import sys
j = json.load(open(sys.argv[1]))
for [[v1, v2], _] in j["dcs"]:
	p1 = j["vs"][str(v1)]["pos"]
	p2 = j["vs"][str(v2)]["pos"]
