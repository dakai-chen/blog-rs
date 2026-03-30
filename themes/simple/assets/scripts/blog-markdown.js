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
