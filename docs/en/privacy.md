# Privacy

Русская версия: [docs/ru/privacy.md](../ru/privacy.md)

A voice profile is biometric data: it describes a person's voice well enough to tell them apart
from others. The app is built around that fact.

## What the app does with your voice

| Data | Where it lives | How long |
| --- | --- | --- |
| Audio from the microphone or from files | In memory | Until it is processed, then dropped |
| A draft profile (vectors) | In memory | Until you save it or close the window |
| A saved profile (`.qvoice.json`) | The app's data folder on your disk | Until you delete it |
| An exported profile | Wherever you saved it | Your responsibility |

- **Audio is never written to disk**, not even temporarily.
- **Nothing is sent anywhere.** The app makes no network calls: there is no server, no account,
  no analytics, no update check. The models are inside the installer.
- **A profile is created only by your action:** recording, then pressing **Save profile**.
- **Deleting a profile deletes its file.**

## What is in a profile file

Lists of numbers (speaker vectors), the name you typed, the creation time and the identifier of
the model. No audio and no text of what was said. Speech cannot be played back from the vectors,
but they do identify a voice, so treat the file like a personal document.

## A voice is not a password

Voice recognition here answers "who is probably speaking", not "is this really them". A
recording of the person, or a synthetic copy of their voice, can score as high as the person
does. Do not use a voice match as the only condition for anything that needs protection:
payments, unlocking doors, access to private data.

## Recording other people

Create a profile for another person only with their consent, and tell them what the file is for.
Laws on biometric data differ between countries, and the responsibility for following them is
yours.

## Handing the file over

Move the exported file into Quant-Core directly, over a channel you trust. Do not commit it to a
repository: `*.qvoice.json` is in this project's `.gitignore` for that reason.
