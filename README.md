# 🐺 Fenrir

**Fenrir** é um engine de imagens de alta performance escrito em Rust, com bindings para Python, pensado para trabalhar com **arquivos gigantes** (TIFF, BigTIFF, PSD, etc) sem sofrer.

> Unchain your images.

---

## ✨ Visão

O objetivo do Fenrir é ser um sucessor moderno do Pillow para uso em projetos mais pesados:

- Processamento de imagens em **gigabytes**
- Suporte a formatos profissionais (TIFF, BigTIFF, PSD)
- Uso de **Rust**, **paralelismo** e **SIMD**
- Bindings limpos para Python via `pyo3` + `maturin`

No começo, o foco é ser **pequeno, simples e sólido**:
ler arquivos, inspecionar metadados e fazer operações básicas de forma segura e rápida.

---

## 🚧 Status

🚨 **Projeto em fase inicial / experimental.**

- [x] Estrutura mínima em Rust
- [x] Binding básico para Python (`fenrir.hello()`)
- [ ] Ler imagens reais (TIFF/PSD)
- [ ] Resize em tiles
- [ ] Integração com NumPy
- [ ] Suporte a PSD multi-camadas

---

## 🔧 Instalação (desenvolvimento)

Por enquanto, o Fenrir ainda não está publicado no PyPI.  
Para testar localmente:

```bash
# Clonar o projeto
git clone https://github.com/SEU-USUARIO/fenrir.git
cd fenrir

# Instalar dependências de build
pip install maturin

# Compilar e instalar no ambiente atual
maturin develop
