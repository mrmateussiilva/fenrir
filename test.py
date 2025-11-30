import fenrir
import time

img = fenrir.FenrirImage.open("images/image.jpeg")
img.show_viewer()          # dispara o viewer em outra thread
# mantém o script rodando
input("Viewer aberto – pressione Enter para sair...")
