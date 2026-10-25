# 0038 Mail files open and download

Date: 2026-10-21. Status: accepted.

A file in a mail reader opens on a press (`on_open_file`, its message's key and the file's place) and its button downloads a file not yet here or saves one that is (`on_download`, tipped Save As…); both name a file by its place, as two files may share a name. `FileMessage::on_open` makes the file a button; `download_tip` names its button.
