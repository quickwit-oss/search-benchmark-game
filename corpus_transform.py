import fileinput
import json
import random
import re

PTN = re.compile("[^a-zA-Z]+")

random.seed(42)

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
        "text": transform(doc["body"]),
        "sort_field": random.randint(0, 2**32 - 1)
    }

    print(json.dumps(doc_transformed))
