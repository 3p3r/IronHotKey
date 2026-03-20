#SingleInstance Force

MonitorGetCount()
MonitorGetPrimary()
MonitorGetName()
MonitorGet()
MonitorGetWorkArea()
SysGet("MonitorCount")
SysGet("MonitorPrimary")
SysGet("0")
SysGet("1")
PixelGetColor(0, 0, "RGB")
PixelSearch(0, 0, 1, 1, "0x000000", 0)
ImageSearch(0, 0, 1, 1, "/nonexistent.png")
