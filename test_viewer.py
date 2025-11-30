import os

import fenrir


def test_show_viewer(monkeypatch):
    monkeypatch.setenv("FENRIR_DISABLE_VIEWER", "1")
    img = fenrir.FenrirImage.new(4, 4, "RGB", (0, 0, 0, 255))
    img.show_viewer()  # should not raise
