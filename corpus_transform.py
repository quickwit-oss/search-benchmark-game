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

    doc_transformed = {
        "id": doc["url"],
        "text": transform(doc["body"])
    }

    print(json.dumps(doc_transformed))
