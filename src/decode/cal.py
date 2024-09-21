import math

fout = open("out.txt", mode='w')

fout.write("[\n\t[\n\t\t")

for i in range(0, 36):
    fout.write(str(math.sin(math.pi / 36.0 * (i + 0.5))))
    fout.write(", ")
fout.write("\n\t],[\n\t\t")

for i in range(0, 18):
    fout.write(str(math.sin(math.pi / 36.0 * (i + 0.5))))
    fout.write(", ")
for i in range(18, 24):
    fout.write(str(1.0))
    fout.write(", ")
for i in range(24, 30):
    fout.write(str(math.sin(math.pi / 12.0 * (i - 18.0 + 0.5))))
    fout.write(", ")
for i in range(30, 36):
    fout.write(str(0.0))
    fout.write(", ")
fout.write("\n\t],[\n\t\t")

for i in range(0, 12):
    fout.write(str(math.sin(math.pi / 12.0 * (i + 0.5))))
    fout.write(", ")
for i in range(12, 36):
    fout.write(str(0.0))
    fout.write(", ")
fout.write("\n\t],[\n\t\t")

for i in range(0, 6):
    fout.write("0.0, ")
for i in range(6, 12):
    fout.write(str(math.sin(math.pi / 12.0 * (i - 6.0 + 0.5))))
    fout.write(", ")
for i in range(12, 18):
    fout.write(str(1.0))
    fout.write(", ")
for i in range(18, 36):
    fout.write(str(math.sin(math.pi / 36.0 * (i + 0.5))))
    fout.write(", ")
fout.write("\n\t]\n]\n")