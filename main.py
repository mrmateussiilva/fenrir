import fenrir

img = fenrir.FenrirImage.open("images/image.jpeg")
img.to_ascii()
print("SIZE:", img.get_size())

new_img = img.crop(0, 0, 300, 300)
print("AFTER CROP:", img.get_size())
new_img.save("images/teste.jpeg")
img.resize(500, 500)
print("AFTER RESIZE:", img.get_size())

img.save("out.jpeg")
