#SingleInstance Force

Process("Exist")
Process("Exist", "999999999")
Run("echo hello")
RunWait("echo hello")
RunAs()
