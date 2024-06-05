# 0.2.0
- Moved to `windows-sys` for the Win32 APIs
- Removed the 255 character restriction (not present in recent builds, will just silently truncate on the Delphi end if limited)
    - New limit is 4096 characters due to the static buffer
    - `ShortString` -> `AnsiString` to reflect the changes
- `SmartieInfo` and `SmartieDemo` now shows specific errors instead of a generic error.