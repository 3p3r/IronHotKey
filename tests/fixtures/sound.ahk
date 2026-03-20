#SingleInstance Force

SoundBeep(523, 50)
SoundBeep()
SoundPlay("*-1")
SoundPlay("*16")
SoundGet("MASTER", "VOLUME")
SoundGet("MASTER", "MUTE")
SoundSet("50", "MASTER", "VOLUME")
SoundGetWaveVolume()
SoundSetWaveVolume("50")
