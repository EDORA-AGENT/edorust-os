
import os
import shutil
import struct
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
IMAGE = os.path.join(ROOT, "build", "edora-nova-memory-map.img")
KERNEL = os.path.join(
    ROOT, "target", "x86_64-unknown-none", "release", "edora-nova"
)
BACKUP = IMAGE + ".bak"

PARTITION_LBA = 128
MASK = 0x0FFFFFFF
EOC = 0x0FFFFFFF


def u16(data, offset):
    return struct.unpack_from("<H", data, offset)[0]


def u32(data, offset):
    return struct.unpack_from("<I", data, offset)[0]


def put32(data, offset, value):
    struct.pack_into("<I", data, offset, value)


def main():
    if not os.path.isfile(IMAGE) or not os.path.isfile(KERNEL):
        raise RuntimeError("Khong tim thay image hoac kernel.")

    kernel = open(KERNEL, "rb").read()
    if not kernel:
        raise RuntimeError("Kernel rong.")

    if not os.path.isfile(BACKUP):
        shutil.copy2(IMAGE, BACKUP)
        print("Backup:", BACKUP)
    else:
        print("Backup da ton tai:", BACKUP)

    with open(IMAGE, "r+b") as f:
        data = bytearray(f.read())

        part_off = PARTITION_LBA * 512
        boot = data[part_off:part_off + 512]
        if boot[510:512] != b"\x55\xaa":
            raise RuntimeError("Boot sector khong hop le.")

        bps = u16(boot, 11)
        spc = boot[13]
        reserved = u16(boot, 14)
        fats = boot[16]
        spf = u32(boot, 36)
        root = u32(boot, 44)

        if bps != 512 or spc == 0 or fats != 2:
            raise RuntimeError("Thong so FAT32 khong dung nhu du kien.")

        csize = bps * spc
        fat_start = part_off + reserved * bps
        data_start = fat_start + fats * spf * bps

        def coff(c):
            return data_start + (c - 2) * csize

        def fat_get(c):
            return u32(data, fat_start + c * 4) & MASK

        def fat_set(c, value):
            for i in range(fats):
                off = (
                    part_off + reserved * bps
                    + i * spf * bps + c * 4
                )
                old = u32(data, off)
                put32(data, off, (old & 0xF0000000) | value)

        def get_chain(start):
            chain = []
            seen = set()
            c = start
            while c >= 2:
                if c in seen:
                    raise RuntimeError("Phat hien vong lap FAT.")
                seen.add(c)
                chain.append(c)
                nxt = fat_get(c)
                if nxt >= 0x0FFFFFF8:
                    return chain
                if nxt < 2 or nxt == 0x0FFFFFF7:
                    raise RuntimeError("Chuoi cluster FAT bi hong.")
                c = nxt
            raise RuntimeError("Chuoi cluster khong hop le.")

        def directory_entries(start):
            for c in get_chain(start):
                off = coff(c)
                for pos in range(0, csize, 32):
                    e = data[off + pos:off + pos + 32]
                    if len(e) < 32 or e[0] == 0:
                        return
                    if e[0] == 0xE5 or e[11] == 0x0F:
                        continue
                    if e[11] & 0x08:
                        continue
                    name = e[:8].decode("ascii", "replace").rstrip()
                    ext = e[8:11].decode("ascii", "replace").rstrip()
                    full = name + ("." + ext if ext else "")
                    high, low = u16(e, 20), u16(e, 26)
                    cluster = (high << 16) | low
                    size = u32(e, 28)
                    yield full, cluster, size, off + pos

        boot_cluster = None
        for name, cluster, _, _ in directory_entries(root):
            if name.upper() == "BOOT":
                boot_cluster = cluster
                break

        if boot_cluster is None:
            raise RuntimeError("Khong tim thay thu muc BOOT.")

        matches = [
            entry for entry in directory_entries(boot_cluster)
            if entry[0].upper().endswith(".ELF")
        ]
        if len(matches) != 1:
            raise RuntimeError(
                "Can dung mot file ELF trong BOOT, tim thay: "
                + str([x[0] for x in matches])
            )

        name, first_cluster, old_size, entry_off = matches[0]
        old_chain = get_chain(first_cluster)
        needed = (len(kernel) + csize - 1) // csize

        print("Kernel trong image:", name)
        print("Kich thuoc cu:", old_size)
        print("Kich thuoc moi:", len(kernel))
        print("Cluster cu:", len(old_chain))
        print("Cluster can:", needed)

        if needed > len(old_chain):
            total_clusters = (len(data) - data_start) // csize + 2
            free = []
            for c in range(2, total_clusters):
                if c not in old_chain and fat_get(c) == 0:
                    free.append(c)
                    if len(old_chain) + len(free) == needed:
                        break

            if len(old_chain) + len(free) < needed:
                raise RuntimeError("Khong du cluster trong.")

            new_chain = old_chain + free
        else:
            new_chain = old_chain[:needed]

        # Ghi kernel vào chuỗi cluster đã xác định.
        for i, c in enumerate(new_chain):
            chunk = kernel[i * csize:(i + 1) * csize]
            chunk = chunk.ljust(csize, b"\x00")
            off = coff(c)
            data[off:off + csize] = chunk

        # Cập nhật liên kết FAT cho chuỗi kernel.
        for i, c in enumerate(new_chain):
            fat_set(c, new_chain[i + 1] if i + 1 < len(new_chain) else EOC)

        # Nếu chuỗi cũ dài hơn cần thiết, giải phóng cluster thừa.
        for c in old_chain[needed:]:
            fat_set(c, 0)

        # Cập nhật kích thước file trong directory entry.
        put32(data, entry_off + 28, len(kernel))

        # Chỉ ghi image sau khi toàn bộ thay đổi đã được chuẩn bị.
        with open(IMAGE, "r+b") as out:
            out.seek(0)
            out.write(data)
            out.flush()
            os.fsync(out.fileno())

    print("Cap nhat xong:", IMAGE)
    print("Hay chay inspect_image.py de xac minh truoc khi boot.")


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        print("ERROR:", exc)
        sys.exit(1)
