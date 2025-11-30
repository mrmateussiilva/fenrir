# fenrir.pyi
# Tipagem oficial do Fenrir Engine
# Garante autocomplete e IntelliSense nas IDEs

from typing import List, Tuple


class FenrirImage:
    """
    FenrirImage representa uma imagem carregada na memória.

    Ela fornece operações básicas de manipulação:
    - abrir
    - salvar
    - cortar (crop)
    - redimensionar (resize)
    - dividir por segmentos (split)

    OBS: A implementação real é feita em Rust via pyo3.
    """

    width: int
    height: int

    @staticmethod
    def new(width: int, height: int, mode: str, color: Tuple[int, int, int, int]) -> "FenrirImage":
        """
        Cria uma nova imagem preenchida com a cor desejada.

        Args:
            width: Largura em pixels (> 0).
            height: Altura em pixels (> 0).
            mode: "RGB", "RGBA" ou "L".
            color: RGBA usado para preencher o plano inicial.

        Returns:
            Instância pronta para desenho:

            >>> img = FenrirImage.new(128, 128, "RGBA", (0, 0, 0, 255))
            >>> img.fill((255, 0, 0, 255))
        """
        ...

    @staticmethod
    def open(path: str) -> "FenrirImage":
        """
        Abre uma imagem do disco e retorna uma instância FenrirImage.

        Args:
            path: Caminho para o arquivo (PNG, JPEG, WEBP, BMP, etc.)

        Returns:
            FenrirImage: A imagem carregada.

        Raises:
            IOError: Se o arquivo não existir ou não puder ser lido.
        """
        ...

    def save(self, path: str) -> None:
        """
        Salva a imagem atual no caminho especificado.

        Args:
            path: Caminho onde o arquivo será salvo.

        Raises:
            IOError: Se ocorrer um erro ao salvar.
        """
        ...

    def get_size(self) -> Tuple[int, int]:
        """
        Retorna as dimensões (largura, altura) da imagem.

        Returns:
            (width, height)
        """
        ...

    def get_pixel(self, x: int, y: int) -> Tuple[int, int, int, int]:
        """
        Lê a cor (RGBA) de um pixel específico.

        Args:
            x: Coluna alvo.
            y: Linha alvo.
        """
        ...

    def crop(self, x: int, y: int, w: int, h: int) -> "FenrirImage":
        """
        Corta a imagem para a região especificada.

        Args:
            x: Posição X inicial.
            y: Posição Y inicial.
            w: Largura do corte.
            h: Altura do corte.

        Returns:
            None
        """
        ...

    def resize(self, width: int, height: int) -> None:
        """
        Redimensiona a imagem usando filtro Lanczos3.

        Args:
            width: Nova largura.
            height: Nova altura.

        Returns:
            None
        """
        ...

    def split(self, cuts: List[int]) -> List["FenrirImage"]:
        """
        Divide a imagem em múltiplos segmentos, baseado nos cortes fornecidos.

        A sintaxe é:
            [axis, pos1, pos2, pos3, ...]

        - axis = 0 → cortes verticais
        - axis = 1 → cortes horizontais

        Exemplo:
            img.split([0, 10, 30, 40])
            → retorna 4 segmentos verticais

            img.split([1, 50, 200])
            → retorna 3 segmentos horizontais

        Args:
            cuts: Lista contendo eixo e posições de corte.

        Returns:
            Lista de FenrirImage com cada pedaço recortado.

        Raises:
            ValueError: se o vetor for inválido.
        """
        ...

    def draw_pixel(self, x: int, y: int, color: Tuple[int, int, int, int]) -> None:
        """
        Desenha um único pixel validando limites.
        """
        ...

    def draw_line(
        self,
        x1: int,
        y1: int,
        x2: int,
        y2: int,
        color: Tuple[int, int, int, int],
    ) -> None:
        """
        Desenha uma linha usando Bresenham.

        Exemplo::

            img.draw_line(0, 0, 127, 127, (0, 0, 0, 255))
        """
        ...

    def draw_rect(
        self,
        x: int,
        y: int,
        w: int,
        h: int,
        color: Tuple[int, int, int, int],
        fill: bool = True,
    ) -> None:
        """
        Desenha um retângulo preenchido ou apenas contorno.
        """
        ...

    def fill(self, color: Tuple[int, int, int, int]) -> None:
        """
        Preenche toda a imagem com a cor especificada.
        """
        ...

    def linear_gradient(
        self,
        direction: str,
        color_start: Tuple[int, int, int, int],
        color_end: Tuple[int, int, int, int],
    ) -> None:
        """
        Cria um gradiente linear horizontal ou vertical.
        """
        ...

    def show(self) -> None:
        """
        Abre a imagem em um visualizador externo (usa xdg-open/open/start).

        Útil para depuração rápida:
        >>> fenrir.FenrirImage.open("foto.png").show()
        """
        ...

    def to_ascii(self, width: int) -> str:
        """
        Converte a imagem para arte ASCII proporcional à largura informada.
        """
        ...

    def rotate_90(self) -> None:
        """
        Rotaciona a imagem atual em 90 graus.
        """
        ...

    def rotate_180(self) -> None:
        """
        Rotaciona a imagem em 180 graus.
        """
        ...

    def rotate_270(self) -> None:
        """
        Rotaciona a imagem em 270 graus.
        """
        ...

    def show_viewer(self) -> None:
        """
        Abre uma janela estilo Photoshop exibindo a imagem com zoom/pan
        e uma barra lateral de ferramentas para pré-visualização.
        """
        ...
