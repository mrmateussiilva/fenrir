# 📋 Documentação do Projeto Fenrir

## Visão Geral

**Fenrir** é um mecanismo de processamento de imagens de alta performance escrito em Rust com bindings para Python (via PyO3 + Maturin). O objetivo é ser um sucessor moderno do Pillow para projetos mais pesados, com suporte a arquivos gigantes (TIFF, BigTIFF, PSD).

---

## ✅ O que já existe (v0.1.9)

### 1. Core - `src/image.rs` (900+ linhas)

#### Funcionalidades Implementadas:

| Método | Descrição |
|--------|-----------|
| `new(width, height, mode, color)` | Cria imagem vazia (RGB, RGBA, L) |
| `open(path)` | Abre imagem do disco |
| `save(path)` | Salva imagem em disco |
| `get_size()` | Retorna (largura, altura) |
| `get_pixel(x, y)` | Retorna cor RGBA de um pixel |
| `crop(x, y, w, h)` | Corta a imagem para região especificada |
| `resize(w, h)` | Redimensiona usando filtro Lanczos3 |
| `split(cuts)` | Divide em segmentos verticais/horizontais |
| `draw_pixel(x, y, color)` | Desenha pixel |
| `draw_line(x1, y1, x2, y2, color)` | Desenha linha (Bresenham) |
| `draw_rect(x, y, w, h, color, fill)` | Desenha retângulo |
| `fill(color)` | Preenche imagem inteira |
| `linear_gradient(direction, start, end)` | Gradiente linear |
| `rotate_90/180/270()` | Rotaciona 90/180/270 graus |
| `to_ascii(width)` | Converte para arte ASCII |
| `show()` | Abre no visualizador do sistema |
| `show_viewer()` | Abre viewer interativo estilo Photoshop |
| **`tile(tile_width, tile_height)`** | Divide imagem em tiles |
| **`tile_count(tile_width, tile_height)`** | Retorna contagem de tiles |
| **`load_tile(path, x, y, w, h)`** | Carrega tile específico sem carregar imagem inteira |
| **`assemble(tiles)`** | Monta imagem a partir de tiles |

### 2. FenrirTile - `src/image.rs`

| Método | Descrição |
|--------|-----------|
| `new(image, x, y, w, h, orig_w, orig_h)` | Construtor |
| `tile_x/y/width/height()` | Getters para posição e dimensões |
| `get_original_size()` | Dimensões da imagem original |
| `get_image()` | Retorna a imagem do tile |
| `save(path)` | Salva tile em disco |
| `get_absolute_position()` | Posição absoluta na imagem original |
| `apply(operation, params)` | Aplica operação (invert, brightness, grayscale) |

---

### 2. Viewer Interativo - `src/viewer.rs` (731 linhas)

UI desktop estilo Photoshop usando **egui/eframe**:

#### Ferramentas Implementadas:
- **Crop** - Cortar região
- **Resize** - Redimensionar
- **Split** - Dividir imagem
- **Rotate** - Rotacionar 90/180/270°
- **Draw Pixel** - Desenhar pixel
- **Draw Line** - Desenhar linha
- **Draw Rect** - Desenhar retângulo
- **Fill** - Preencher cor
- **Gradient** - Gradiente linear
- **ASCII Preview** - Visualizar como ASCII

#### Recursos:
- Zoom (scroll) e pan (drag)
- Fundo checkerboard (transparência)
- FPS counter
- Barra lateral com ferramentas
- Suporte a Linux (Wayland/X11), macOS, Windows

---

### 3. Interface Python - `fenrir.pyi` (239 linhas)

Stub de tipagem com documentação completa:
- Todas as classes e métodos documentados
- Exemplos de uso em docstrings
- Type hints completos

---

### 4. Build System

| Arquivo | Descrição |
|---------|-----------|
| `Cargo.toml` | Projeto Rust (cdylib) |
| `pyproject.toml` | Build com Maturin |
| `Cargo.lock` | Dependências travadas |

**Dependências Rust:**
- `pyo3` 0.21 - bindings Python
- `image` 0.24 - processamento de imagens
- `tempfile` 3 - arquivos temporários
- `egui` 0.28 / `eframe` 0.28 - UI
- `winit` 0.29 - windowing

---

### 5. Testes

| Arquivo | Cobertura |
|---------|-----------|
| `test_fenrir_creation.py` | Criação de imagens |
| `test_viewer.py` | Viewer interativo |
| `test_visualization.py` | Visualização |
| `test_split.py` | Divisão de imagens |

---

## ❌ O que falta (Roadmap)

### Alto Prioridade

| # | Item | Descrição |
|---|------|-----------|
| ~~1~~ | ~~Tile-based Processing~~ | ✅ **IMPLEMENTADO** |
| 2 | **Suporte TIFF/BigTIFF** | O README menciona arquivos gigantes, mas ainda não implementado |
| 3 | **Suporte PSD** | Formato Photoshop multi-camadas |
| 4 | **NumPy Integration** | Retornar arrays NumPy diretamente |

### Médio Prioridade

| # | Item | Descrição |
|---|------|-----------|
| 5 | **Mais formatos** | BMP, GIF, TIFF multipágina |
| 6 | **Operações em paralelo** | SIMD/paralelismo com Rayon |
| 7 | **Metadados EXIF** | Ler metadata de imagens |
| 8 | **Operações rápidas** | blur, sharpen, brightness, contrast |

### Baixa Prioridade

| # | Item | Descrição |
|---|------|-----------|
| 9 | **PubPyPI** | Publicar no PyPI |
| 10 | **Documentação Sphinx** | Docs automatizadas |
| 11 | **Dockerfile** | Container pronto |
| 12 | **Benchmarks** | Comparativo com Pillow |

---

## 🔍 Análise Técnica

### Pontos Fortes:
1. API Python limpa e intuitiva
2. Viewer interativo impressionante para debugging
3. Código Rust bem estruturado
4. Tratamento de erros adequado
5. Tipagem completa (.pyi)

### Issues/Atenção:
1. `edition = "2024"` no Cargo.toml - **essa edição ainda não existe** (atualmente 2021 é a última)
2. O nome "Fenrir" para engine de imagens grandes sugere BigTIFF, mas ainda não implementado
3. Não há integração com NumPy ainda (limitação atual)
4. viewer adiciona dependência heavy (egui/eframe/winit) - pode pesar na lib

---

## 📦 Estrutura de Arquivos

```
fenrir/
├── Cargo.toml          # Projeto Rust
├── Cargo.lock          # Dependências
├── pyproject.toml      # Build Python
├── fenrir.pyi          # Tipagem
├── README.md           # Documentação
├── src/
│   ├── lib.rs          # Módulo Python
│   ├── image.rs        # Processamento
│   └── viewer.rs       # UI interativa
└── test_*.py           # Testes
```

---

## 🚀 Como Usar

```python
import fenrir

# Criar
img = fenrir.FenrirImage.new(800, 600, "RGBA", (255, 0, 0, 255))

# Abrir
img = fenrir.FenrirImage.open("foto.png")

# Manipular
img.resize(400, 300)
img.crop(10, 10, 200, 200)
img.draw_line(0, 0, 100, 100, (0, 255, 0, 255))

# Salvar
img.save("output.png")

# Viewer interativo
img.show_viewer()
```

---

## 📊 Status do Projeto

- **Versão:** 0.1.9 (experimental)
- **Estabilidade:** API pode mudar
- **Build:** ✅ Funcionando (local)
- **Testes:** ✅ Existentes
- **Publicação PyPI:** ❌ Não publicado
