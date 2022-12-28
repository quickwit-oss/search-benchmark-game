import fileinput
import json
import re

PTN = re.compile("[^a-zA-Z]+")

def transform(text):
    return PTN.sub(" ", text.lower())

for line in fileinput.input():
    doc = {}
    try:
        doc = json.loads(line)
    except ValueError:
        continue

    if doc["url"] == "":
        continue
    if len(doc["url"].split("curid=",1)) == 1:
        continue

    doc_transformed = {
        "id": doc["url"],
        "id_num": int(doc["url"].split("curid=",1)[1]),
        "text": transform(doc["body"])
    }

    print(json.dumps(doc_transformed))
