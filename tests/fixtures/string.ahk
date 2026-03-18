#SingleInstance Force

; InStr - search for substring
InStr("Hello World", "o")
InStr("Hello World", "World")
InStr("Hello World", "xyz")
InStr("HELLO", "hello", 0)
InStr("HELLO", "hello", 1)
InStr("Hello World", "l", , 4)
InStr("abcabcabc", "bc", , , 2)

; StrLen - string length
StrLen("Hello")
StrLen("")
StrLen("you")

; StrReplace - replace substring
StrReplace("Hello World", "World", "AHK")
StrReplace("aaa", "a", "b")
StrReplace("Hello", "xyz", "test")
StrReplace("test", "t", "T", , 1)
StrReplace("aaaa", "a", "b", , 2)

; SubStr - extract substring
SubStr("Hello World", 1, 5)
SubStr("Hello World", 7)
SubStr("Hello", 2, 3)
SubStr("Hello", -2)
SubStr("Hello", 100)
SubStr("Hello", 0)

; Trim/LTrim/RTrim - trim whitespace
Trim("  Hello  ")
Trim("xxxHelloxxx", "x")
LTrim("  Hello")
LTrim("xxxHello", "x")
RTrim("Hello  ")
RTrim("Helloxxx", "x")

; RegExMatch - regex matching
RegExMatch("Hello123World", "\\d+")
RegExMatch("test", "test")
RegExMatch("Hello", "xyz")
RegExMatch("HELLO", "i)hello")

; RegExReplace - regex replacement
RegExReplace("test123test", "\\d+", "X")
RegExReplace("HELLO", "o", "O")
RegExReplace("no match", "xyz", "abc")

; String deprecated functions via function syntax
StringLen("Hello")
StringUpper("hello")
StringLower("HELLO")
StringTrimLeft("Hello World", 6)
StringTrimRight("Hello World", 5)
StringLeft("Hello", 2)
StringRight("Hello", 3)
StringMid("Hello World", 7, 5)
StringMid("Hello", 2, 2)
StringGetPos("Hello World", "World")
StringGetPos("Hello World", "xyz")
StringReplace("Hello World", "World", "AHK")
StringReplace("test", "t", "T")

