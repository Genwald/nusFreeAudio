# nusFreeAudio
Free yourself from the nus3audio format.

Creates a nus3audio file from your audio files at runtime.  
Place files at `rom:/nusFreeAudio/[arc path].nus3audio/[files here]`

Depends on the arcropolis 0.9.6 api. Acropolis 0.9.6 does not support callbacks for stream files.

As a reminder the game appears to support the following formats by default:
- WAV
- LOPUS
- OPUS
- IDSP
- DSP
- CAF
- IS14
- IMRV