import pytest
import fenrir
import tempfile
import os


def test_tiff_writer_create():
    """Testa criação de TIFF writer."""
    with tempfile.NamedTemporaryFile(suffix=".tif", delete=False) as f:
        temp_path = f.name

    try:
        writer = fenrir.FenrirTiffWriter(temp_path, 100, 100)
        w, h = writer.get_size()
        assert w == 100
        assert h == 100
    finally:
        if os.path.exists(temp_path):
            os.unlink(temp_path)


def test_tiff_writer_set_pixel():
    """Testa definir pixel no TIFF."""
    with tempfile.NamedTemporaryFile(suffix=".tif", delete=False) as f:
        temp_path = f.name

    try:
        writer = fenrir.FenrirTiffWriter(temp_path, 10, 10)
        writer.set_pixel(0, 0, (255, 0, 0, 255))
        writer.save()

        assert os.path.exists(temp_path)
    finally:
        if os.path.exists(temp_path):
            os.unlink(temp_path)


def test_tiff_writer_fill():
    """Testa preencher TIFF com cor."""
    with tempfile.NamedTemporaryFile(suffix=".tif", delete=False) as f:
        temp_path = f.name

    try:
        writer = fenrir.FenrirTiffWriter(temp_path, 10, 10)
        writer.fill((0, 255, 0, 255))
        writer.save()

        assert os.path.exists(temp_path)

        img = fenrir.FenrirImage.open(temp_path)
        pixel = img.get_pixel(0, 0)
        assert pixel[0] == 0
        assert pixel[1] == 255
    finally:
        if os.path.exists(temp_path):
            os.unlink(temp_path)


def test_tiff_writer_to_fenrir():
    """Testa converter TIFF para FenrirImage."""
    with tempfile.NamedTemporaryFile(suffix=".tif", delete=False) as f:
        temp_path = f.name

    try:
        writer = fenrir.FenrirTiffWriter(temp_path, 50, 50)
        writer.fill((100, 150, 200, 255))
        writer.save()

        tiff = fenrir.FenrirTiff(temp_path)
        w, h = tiff.get_size()
        assert w == 50
        assert h == 50

        img = tiff.to_fenrir_image()
        assert img is not None
    finally:
        if os.path.exists(temp_path):
            os.unlink(temp_path)


def test_tiff_load_region():
    """Testa carregar região do TIFF."""
    with tempfile.NamedTemporaryFile(suffix=".tif", delete=False) as f:
        temp_path = f.name

    try:
        writer = fenrir.FenrirTiffWriter(temp_path, 100, 100)
        writer.fill((255, 255, 255, 255))
        writer.save()

        tiff = fenrir.FenrirTiff(temp_path)
        region = tiff.load_region(10, 10, 50, 50)

        w, h = region.get_size()
        assert w == 50
        assert h == 50
    finally:
        if os.path.exists(temp_path):
            os.unlink(temp_path)


def test_tiff_invalid_extension():
    """Testa erro com extensão inválida."""
    with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as f:
        temp_path = f.name

    try:
        with pytest.raises(ValueError, match="deve ser TIFF"):
            fenrir.FenrirTiff(temp_path)
    finally:
        if os.path.exists(temp_path):
            os.unlink(temp_path)


def test_tiff_nonexistent_file():
    """Testa erro com arquivo inexistente."""
    with pytest.raises(IOError, match="não encontrado"):
        fenrir.FenrirTiff("/tmp/não_existe.tif")
