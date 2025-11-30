from pathlib import Path

import pytest

import fenrir


def write_sample_image(tmp_path: Path) -> Path:
    path = tmp_path / "sample.ppm"
    rows = [
        " ".join(["255 0 0"] * 6),
        " ".join(["0 255 0"] * 6),
        " ".join(["0 0 255"] * 6),
        " ".join(["255 255 0"] * 6),
    ]
    ppm_data = "P3\n6 4\n255\n" + "\n".join(rows) + "\n"
    path.write_text(ppm_data)
    return path


def load_test_image(tmp_path: Path) -> fenrir.FenrirImage:
    img_path = write_sample_image(tmp_path)
    image = fenrir.FenrirImage.open(str(img_path))
    image.resize(6, 4)
    return image


def test_split_vertical(tmp_path):
    image = load_test_image(tmp_path)
    parts = image.split([0, 2, 4])

    assert len(parts) == 3
    assert [part.get_size() for part in parts] == [(2, 4), (2, 4), (2, 4)]


def test_split_horizontal(tmp_path):
    image = load_test_image(tmp_path)
    parts = image.split([1, 3])

    assert len(parts) == 2
    assert [part.get_size() for part in parts] == [(6, 3), (6, 1)]


@pytest.mark.parametrize("payload", [[], [5]])
def test_split_invalid_input(tmp_path, payload):
    image = load_test_image(tmp_path)

    with pytest.raises(ValueError):
        image.split(payload)
