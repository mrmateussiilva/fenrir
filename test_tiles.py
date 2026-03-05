import pytest
import fenrir


def test_tile_creation():
    """Testa criação de tiles a partir de uma imagem."""
    img = fenrir.FenrirImage.new(100, 100, "RGBA", (255, 0, 0, 255))

    tiles = img.tile(50, 50)

    assert len(tiles) == 4
    assert tiles[0][0] == 0
    assert tiles[0][1] == 0
    assert tiles[0][2].tile_width() == 50
    assert tiles[0][2].tile_height() == 50


def test_tile_count():
    """Testa contagem de tiles."""
    img = fenrir.FenrirImage.new(100, 100, "RGBA", (255, 0, 0, 255))

    cols, rows = img.tile_count(50, 50)

    assert cols == 2
    assert rows == 2


def test_tile_count_uneven():
    """Testa contagem com dimensões não evenly divisíveis."""
    img = fenrir.FenrirImage.new(100, 100, "RGBA", (255, 0, 0, 255))

    cols, rows = img.tile_count(30, 40)

    assert cols == 4
    assert rows == 3


def test_tile_get_image():
    """Testa extrair imagem de um tile."""
    img = fenrir.FenrirImage.new(100, 100, "RGBA", (255, 0, 0, 255))

    tiles = img.tile(50, 50)
    col, row, tile = tiles[0]

    tile_img = tile.get_image()
    w, h = tile_img.get_size()

    assert w == 50
    assert h == 50


def test_tile_original_size():
    """Testa obter tamanho original do tile."""
    img = fenrir.FenrirImage.new(100, 100, "RGBA", (255, 0, 0, 255))

    tiles = img.tile(50, 50)
    col, row, tile = tiles[0]

    orig_w, orig_h = tile.get_original_size()

    assert orig_w == 100
    assert orig_h == 100


def test_tile_absolute_position():
    """Testa posição absoluta do tile."""
    img = fenrir.FenrirImage.new(100, 100, "RGBA", (255, 0, 0, 255))

    tiles = img.tile(50, 50)

    col, row, tile = tiles[0]
    x, y = tile.get_absolute_position()
    assert x == 0
    assert y == 0

    col, row, tile = tiles[1]
    x, y = tile.get_absolute_position()
    assert x == 50
    assert y == 0

    col, row, tile = tiles[2]
    x, y = tile.get_absolute_position()
    assert x == 0
    assert y == 50


def test_tile_apply_invert():
    """Testa aplicar inversão de cores em tile."""
    img = fenrir.FenrirImage.new(10, 10, "RGB", (255, 0, 0, 255))

    tiles = img.tile(10, 10)
    col, row, tile = tiles[0]

    inverted = tile.apply("invert", [])

    pixel = inverted.get_image().get_pixel(0, 0)
    assert pixel[0] == 0
    assert pixel[1] == 255
    assert pixel[2] == 255


def test_tile_apply_brightness():
    """Testa aplicar brilho em tile."""
    img = fenrir.FenrirImage.new(10, 10, "RGB", (100, 100, 100, 255))

    tiles = img.tile(10, 10)
    col, row, tile = tiles[0]

    bright = tile.apply("brightness", [2.0])

    pixel = bright.get_image().get_pixel(0, 0)
    assert pixel[0] == 200
    assert pixel[1] == 200
    assert pixel[2] == 200


def test_tile_apply_grayscale():
    """Testa converter tile para escala de cinza."""
    img = fenrir.FenrirImage.new(10, 10, "RGB", (255, 0, 0, 255))

    tiles = img.tile(10, 10)
    col, row, tile = tiles[0]

    gray = tile.apply("grayscale", [])

    pixel = gray.get_image().get_pixel(0, 0)
    assert pixel[0] == pixel[1] == pixel[2]


def test_tile_save():
    """Testa salvar tile em disco."""
    import tempfile
    import os

    img = fenrir.FenrirImage.new(100, 100, "RGBA", (255, 0, 0, 255))

    tiles = img.tile(50, 50)
    col, row, tile = tiles[0]

    with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as f:
        temp_path = f.name

    try:
        tile.save(temp_path)
        assert os.path.exists(temp_path)
    finally:
        os.unlink(temp_path)


def test_load_tile():
    """Testa carregar tile diretamente de arquivo."""
    import tempfile
    import os

    img = fenrir.FenrirImage.new(100, 100, "RGBA", (255, 0, 0, 255))

    with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as f:
        temp_path = f.name

    try:
        img.save(temp_path)

        tile = fenrir.load_tile(temp_path, 0, 0, 50, 50)

        assert tile.tile_x() == 0
        assert tile.tile_y() == 0
        assert tile.tile_width() == 50
        assert tile.tile_height() == 50

        orig_w, orig_h = tile.get_original_size()
        assert orig_w == 100
        assert orig_h == 100
    finally:
        os.unlink(temp_path)


def test_assemble():
    """Testa montar imagem a partir de tiles."""
    import tempfile
    import os

    img1 = fenrir.FenrirImage.new(50, 50, "RGB", (255, 0, 0, 255))
    img2 = fenrir.FenrirImage.new(50, 50, "RGB", (0, 255, 0, 255))
    img3 = fenrir.FenrirImage.new(50, 50, "RGB", (0, 0, 255, 255))
    img4 = fenrir.FenrirImage.new(50, 50, "RGB", (255, 255, 0, 255))

    tiles = [
        (0, 0, fenrir.FenrirTile(img1, 0, 0, 50, 50, 100, 100)),
        (1, 0, fenrir.FenrirTile(img2, 1, 0, 50, 50, 100, 100)),
        (0, 1, fenrir.FenrirTile(img3, 0, 1, 50, 50, 100, 100)),
        (1, 1, fenrir.FenrirTile(img4, 1, 1, 50, 50, 100, 100)),
    ]

    result = fenrir.assemble(tiles)

    w, h = result.get_size()
    assert w == 100
    assert h == 100

    assert result.get_pixel(0, 0)[:3] == (255, 0, 0)
    assert result.get_pixel(99, 0)[:3] == (0, 255, 0)
    assert result.get_pixel(0, 99)[:3] == (0, 0, 255)
    assert result.get_pixel(99, 99)[:3] == (255, 255, 0)


def test_tile_workflow():
    """Testa workflow completo: dividir, processar, montar."""
    import tempfile
    import os

    img = fenrir.FenrirImage.new(100, 100, "RGB", (255, 255, 255, 255))

    tiles = img.tile(50, 50)

    processed_tiles = []
    for col, row, tile in tiles:
        inverted = tile.apply("invert", [])
        processed_tiles.append((col, row, inverted))

    result = fenrir.assemble(processed_tiles)

    w, h = result.get_size()
    assert w == 100
    assert h == 100

    pixel = result.get_pixel(0, 0)
    assert pixel[:3] == (0, 0, 0)
