qemu-img convert -f raw -O qcow2 \
target/x86_64-blog_os/debug/bootimage-blog_os.bin \
build/image.qcow2

VirtualBoxVM --dbg --startvm blog_os