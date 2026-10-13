# Using the app

Русская версия: [docs/ru/usage.md](../ru/usage.md)

## Creating a profile

1. Press the round **+** button, or **New profile** in the sidebar.
2. Type a name in the field that appears in the sidebar and press Enter. Escape cancels.
3. The recording screen opens. The profile is a draft until you save it.

## Recording

1. Press the microphone button and read the text on the screen aloud, in your normal voice and
   at your normal pace.
2. The highlight moves along the line while the microphone hears a voice and stops when you
   pause. It follows a steady reading pace, not the exact words: there is no speech recognition
   in the app, so you do not have to match it.
3. The bars next to the button show how loud your voice is. If they stay flat, the microphone is
   not picking you up.
4. Press the button again to stop. The app cuts out silence, and the counter at the top shows how
   much clean speech you have.

The goal is 30 seconds of clean speech. More helps, up to about a minute. You can record in
several takes: each one is added to the draft.

For a profile that works in real life:

- Record in a quiet room, at the distance you usually speak from.
- Add a second take from another device (a phone, a headset) if the assistant will hear you
  through different microphones. A voice sounds different through each one.
- Speak naturally. A "careful reading" voice differs from the one you use every day.

## Adding files

**Upload files** adds recordings you already have: WAV, MP3, M4A, FLAC, OGG, Opus and WebM,
including voice messages saved from messengers. Several files can be picked at once.

Only the person's own voice should be in the files. Segments that do not sound like the rest are
dropped automatically, and the hint under the microphone says how many, but this only works
while the other voice is a small part of the material.

## Saving

When there is enough speech, **Save profile** appears. Saving turns the draft into a finished
profile: it is written to the app's data folder and marked "ready" in the sidebar.

- A draft is kept in memory only. If you close the window before saving, the recorded speech is
  lost.
- **Start over** clears the speech collected for the draft.
- A saved profile cannot be extended. To improve it, create a new one.

## Checking a profile

Open a saved profile. In the **Check** block, record a new phrase or pick a file. The app shows
two numbers between −1 and 1:

- **Similarity to the profile** compares the phrase with the profile's average vector.
- **Closest segment** compares it with the most similar single segment.

The closer to 1, the more the voice resembles the profile. In our tests the same person through
a different microphone scored about 0.5–0.8 and a different person stayed below about 0.5, but
this depends on the voices and the microphones. The threshold that decides "this is the person"
is set in Quant-Core, not here. Try your own voice and somebody else's to see the gap.

## Export

**Export .qvoice.json** saves a copy of the profile wherever you choose. Load that file into the
person's profile in Quant-Core. The file is biometric data: do not send it through public
channels and do not put it in a repository.

## Deleting

**Delete profile** removes the profile and its file from the app's data folder after a
confirmation. Copies you exported are not touched.

## Language

The switch at the bottom of the sidebar changes the interface and the reading text between
Russian and English. Russian is the default, and the choice is remembered. The language you read
in does not have to match the language you will speak to the assistant in.
