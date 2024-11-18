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

    url = doc["url"]
    if url == "":
        continue

    filters = []
    id_hash = hash(doc["url"])

    if id_hash % 2 == 0:
        filters.append("50%")
    if id_hash % 10 == 3:
        filters.append("10%")
    if id_hash % 100 == 42:
        filters.append("1%")

    doc_transformed = {
        "id": url,
        "text": transform(doc["body"])
    }
    if len(filters) > 0:
        doc_transformed["filter"] = filters

    print(json.dumps(doc_transformed))
