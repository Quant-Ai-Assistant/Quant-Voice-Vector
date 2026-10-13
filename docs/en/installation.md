# Installation

Русская версия: [docs/ru/installation.md](../ru/installation.md)

Installers are on the [Releases](https://github.com/Quant-Ai-Assistant/Quant-Voice-Vector/releases)
page. The app needs no account and no internet connection after it is installed: the models are
inside the installer.

| System | File |
| --- | --- |
| macOS on Apple Silicon (M1 and later) | `..._aarch64.dmg` |
| macOS on Intel | `..._x64.dmg` |
| Windows 10 and 11, 64-bit | `..._x64-setup.exe` |
| Linux, any distribution | `.AppImage` |
| Debian and Ubuntu | `.deb` |

The app is not signed with a paid developer certificate yet, so macOS and Windows show a warning
on the first launch. The steps below get past it.

## macOS

1. Download the `.dmg` for your processor and open it.
2. Drag **Quant Voice Vector** into **Applications**.
3. Start the app. macOS says it "could not verify" the app. Click **Done**.
4. Open **System Settings → Privacy & Security**, scroll down, click **Open Anyway** next to the
   message about Quant Voice Vector and confirm.
5. Allow microphone access when macOS asks.

Step 4 is needed once for every new version. The same from the terminal:

```sh
xattr -dr com.apple.quarantine "/Applications/Quant Voice Vector.app"
```

If you denied microphone access by mistake, turn it on in
**System Settings → Privacy & Security → Microphone**.

## Windows

1. Download `..._x64-setup.exe` and run it.
2. SmartScreen says the app is unrecognized. Click **More info**, then **Run anyway**.
3. Follow the installer.
4. If recording does not start, check **Settings → Privacy & security → Microphone** and allow
   desktop apps to use the microphone.

## Linux

AppImage:

```sh
chmod +x ./*.AppImage
./*.AppImage
```

Debian and Ubuntu:

```sh
sudo apt install ./*.deb
```

The microphone is used through ALSA; PipeWire and PulseAudio work through their ALSA bridge.

## Where data is stored

Only saved profiles are stored, one `.qvoice.json` file per profile. Audio is never written to
disk.

| System | Folder |
| --- | --- |
| macOS | `~/Library/Application Support/ai.quant.voicevector/profiles` |
| Windows | `%APPDATA%\ai.quant.voicevector\profiles` |
| Linux | `~/.local/share/ai.quant.voicevector/profiles` |

## Removal

Delete the app the usual way for your system. Profiles stay in the data folder above: delete
that folder too if you want them gone. Files you exported yourself are wherever you saved them.
