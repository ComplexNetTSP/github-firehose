# TODO 

## 1. Parse PrevData as string 

### Example of correctly parsed commit frame
```json
{
  "received": "2026-05-27T10:48:23.731589386+02:00",
  "repo_commit": {
    "blobs": null,
    "commit": {
      "$link": "bafyreibzywp3wwzihudxtexi5a7msrcdlojn436ulrdicjqht7z6uz2py4"
    },
    "ops": [
      {
        "action": "create",
        "cid": {
          "$link": "bafyreibqus5rq247hbszmwlcfkz2b7iscdjjr7eqdnn5ysi2smhjwjrcxm"
        },
        "path": "app.bsky.feed.like/3mmt4qdgln62j"
      }
    ],
    "prevData": {
      "$link": "bafyreif7lz522gemgdymesliqk5hmoudokt472z3nwd5x2td5vimo55vga"
    },
    "rebase": false,
    "repo": "did:plc:c2siiyz4tk2srtljtrihlm7b",
    "rev": "3mmt4qdtkog2j",
    "seq": 30391757702,
    "since": "3mmt4q7vy3k2u",
    "time": "2026-05-27T08:48:23.072Z",
    "tooBig": false
  }
}
```