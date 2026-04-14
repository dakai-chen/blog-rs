function init_code_copy_btn() {
    document.querySelectorAll('.markdown .code-block-box').forEach(box => {
        const codeBlock = box.querySelector('pre>code');
        const copyBtn = box.querySelector('.code-copy-btn');

        if (copyBtn == null) {
            return;
        }

        copyBtn.addEventListener('click', function () {
            const code = codeBlock.textContent.replace(/\n$/, '');
            navigator.clipboard.writeText(code).then(() => {
                tips_show("tips-item-success", "复制成功");
                // setTimeout(() => { /* 目前不做任何事 */ }, 1000);
            }).catch(err => {
                tips_show("tips-item-error", `复制失败：${err}`);
                // setTimeout(() => { /* 目前不做任何事 */ }, 1000);
            });
        });
    });
}

function init_image_overlay() {
    const overlay = document.createElement('div');
    overlay.className = 'img-overlay';
    const img = document.createElement('img');
    overlay.appendChild(img);
    document.body.appendChild(overlay);

    let scale = 1.0;
    const scaleMin = 0.1;
    const scaleMax = 5;

    let posX = 0, posY = 0;
    let startX = 0, startY = 0;
    let isDragging = false;

    function reset() {
        scale = 1.0;
        posX = 0;
        posY = 0;
        updateTransform();
    }

    function updateTransform() {
        img.style.transform = `translate(${posX}px, ${posY}px) scale(${scale})`;
    }

    // 打开图片
    document.querySelectorAll('.markdown img').forEach(pic => {
        pic.addEventListener('click', () => {
            img.src = pic.src;
            reset();
            overlay.classList.add('show');
            document.body.style.overflow = 'hidden';
        });
    });

    // 滚轮缩放（按比例，不会越放越慢）
    img.addEventListener('wheel', e => {
        e.preventDefault();
        const factor = e.deltaY > 0 ? 0.9 : 1.1;
        scale *= factor;
        scale = Math.max(scaleMin, Math.min(scaleMax, scale));
        updateTransform();
    });

    // 双击：统一还原到原始大小
    img.addEventListener('dblclick', () => {
        reset();
    });

    // 拖动
    img.addEventListener('mousedown', e => {
        isDragging = true;
        startX = e.clientX - posX;
        startY = e.clientY - posY;
        img.style.cursor = 'grabbing';
        e.preventDefault();
    });

    document.addEventListener('mousemove', e => {
        if (!isDragging) return;
        posX = e.clientX - startX;
        posY = e.clientY - startY;
        updateTransform();
    });

    document.addEventListener('mouseup', () => {
        isDragging = false;
        img.style.cursor = 'grab';
    });

    // 关闭
    function close() {
        overlay.classList.remove('show');
        document.body.style.overflow = '';
    }

    overlay.addEventListener('click', e => {
        if (e.target === overlay) close();
    });

    document.addEventListener('keydown', e => {
        if (e.key === 'Escape') close();
    });
}
