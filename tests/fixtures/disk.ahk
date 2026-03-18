#SingleInstance Force

DriveGet("", "List")
DriveSpaceFree("", "/")
FileAppend("hello", "/tmp/ironhotkey_disk_fixture.txt")
FileRead("/tmp/ironhotkey_disk_fixture.txt")
FileReadLine("/tmp/ironhotkey_disk_fixture.txt", 1)
FileExist("/tmp/ironhotkey_disk_fixture.txt")
FileGetAttrib("/tmp/ironhotkey_disk_fixture.txt")
FileGetSize("/tmp/ironhotkey_disk_fixture.txt", "B")
FileGetTime("/tmp/ironhotkey_disk_fixture.txt", "M")
SplitPath("/tmp/ironhotkey_disk_fixture.txt")
LoopFile("/tmp/*", "F")
LoopReadFile("/tmp/ironhotkey_disk_fixture.txt")
