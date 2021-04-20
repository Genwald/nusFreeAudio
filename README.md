# nusFreeAudio
Free yourself from the nus3audio format.

Creates a nus3audio file from your audio files at runtime.  
This means that you can make music mods simply by placing a wav file (or any other supported file type) at the correct path, no conversions to special formats needed.  
## Installation
[Download a release](https://github.com/Genwald/nusFreeAudio/releases/latest) and place the .nro at `sd:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libnusFreeAudio.nro`
### Dependencies - install these before use
- [Skyline](https://github.com/skyline-dev/skyline/releases)
- [Arcropolis](https://github.com/Raytwo/ARCropolis/releases) v1.1.0+ (generally includes Skyline in its releases)
## Usage
Place audio files at `sd:/atmosphere/contents/01006A800016E000/romfs/nusFreeAudio/[arc path].nus3audio/[files here]`  
The game appears to support the following formats by default:
- WAV
- LOPUS
- OPUS
- IDSP
- DSP
- CAF
- IS14
- IMRV