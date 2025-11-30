import fenrir


def test_show(monkeypatch):
    monkeypatch.setenv("FENRIR_SHOW_COMMAND", "/bin/true")
    img = fenrir.FenrirImage.new(3, 3, "RGB", (0, 0, 0, 255))
    img.show()  # should not raise


def test_to_ascii():
    width, height = 20, 10
    target_width = 20
    img = fenrir.FenrirImage.new(width, height, "RGBA", (128, 128, 128, 255))
    ascii_art = img.to_ascii(target_width)

    assert "\n" in ascii_art

    lines = [line for line in ascii_art.strip("\n").split("\n")]
    expected_height = max(1, round((height / width) * target_width))

    assert len(lines) == expected_height
    assert all(len(line) == target_width for line in lines)
