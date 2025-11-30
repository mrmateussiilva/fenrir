from pathlib import Path

import fenrir


def test_create_rgb_image():
    img = fenrir.FenrirImage.new(200, 100, "RGB", (255, 0, 0, 255))
    assert img.get_size() == (200, 100)


def test_draw_pixel(tmp_path):
    img = fenrir.FenrirImage.new(10, 10, "RGBA", (0, 0, 0, 0))
    img.draw_pixel(2, 3, (255, 128, 64, 255))
    out_path = tmp_path / "pixel.png"
    img.save(str(out_path))

    loaded = fenrir.FenrirImage.open(str(out_path))
    assert loaded.get_pixel(2, 3) == (255, 128, 64, 255)


def test_draw_line(tmp_path):
    img = fenrir.FenrirImage.new(20, 20, "RGB", (255, 255, 255, 255))
    img.draw_line(0, 0, 19, 19, (0, 0, 0, 255))
    path = tmp_path / "line.png"
    img.save(str(path))

    loaded = fenrir.FenrirImage.open(str(path))
    assert loaded.get_pixel(0, 0) == (0, 0, 0, 255)
    assert loaded.get_pixel(10, 10) == (0, 0, 0, 255)
    assert loaded.get_pixel(19, 19) == (0, 0, 0, 255)


def test_draw_rect(tmp_path):
    filled = fenrir.FenrirImage.new(20, 20, "RGBA", (0, 0, 0, 0))
    filled.draw_rect(2, 2, 10, 8, (255, 0, 0, 255), fill=True)
    filled_path = tmp_path / "rect_fill.png"
    filled.save(str(filled_path))
    reload_filled = fenrir.FenrirImage.open(str(filled_path))
    assert reload_filled.get_pixel(5, 5) == (255, 0, 0, 255)
    assert reload_filled.get_pixel(1, 1) == (0, 0, 0, 0)

    outline = fenrir.FenrirImage.new(20, 20, "RGBA", (0, 0, 0, 0))
    outline.draw_rect(3, 3, 6, 6, (0, 255, 0, 255), fill=False)
    outline_path = tmp_path / "rect_outline.png"
    outline.save(str(outline_path))
    reload_outline = fenrir.FenrirImage.open(str(outline_path))
    assert reload_outline.get_pixel(3, 3) == (0, 255, 0, 255)
    assert reload_outline.get_pixel(8, 8) == (0, 255, 0, 255)
    assert reload_outline.get_pixel(5, 5) == (0, 0, 0, 0)


def test_fill():
    img = fenrir.FenrirImage.new(5, 5, "RGBA", (10, 20, 30, 40))
    img.fill((200, 150, 100, 255))
    assert all(
        img.get_pixel(x, y) == (200, 150, 100, 255)
        for x in range(5)
        for y in range(5)
    )


def test_linear_gradient(tmp_path):
    img = fenrir.FenrirImage.new(10, 2, "RGB", (0, 0, 0, 255))
    img.linear_gradient("horizontal", (255, 0, 0, 255), (0, 0, 255, 255))
    path = tmp_path / "gradient.png"
    img.save(str(path))

    reload = fenrir.FenrirImage.open(str(path))
    assert reload.get_pixel(0, 0) == (255, 0, 0, 255)
    assert reload.get_pixel(9, 1) == (0, 0, 255, 255)


def test_rotate_methods():
    img = fenrir.FenrirImage.new(3, 5, "RGB", (0, 0, 0, 255))
    img.rotate_90()
    assert img.get_size() == (5, 3)
    img.rotate_180()
    assert img.get_size() == (5, 3)
    img.rotate_270()
    assert img.get_size() == (3, 5)
