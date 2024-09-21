fin = open("tbl.txt", mode='r')
fout = open("out.txt", mode='w')

old_x = ""
for line in fin.readlines():
    x = ""
    bit_len = ""
    value = ""
    count = 0
    flag = 0
    for c in line:
        if c.isspace():
            if flag == 1:
                flag = 0
                count += 1
            continue
        else:
            if count == 0:
                x += c
            elif count == 2:
                bit_len += c
            elif count == 3:
                value += c
            if flag == 0:
                flag = 1
    if old_x != x and old_x != "":
        fout.write("    ],&[\n")
    old_x = x
    fout.write("        Binary { bit_length: " + bit_len + ", value: 0b"+value+"},\n")
