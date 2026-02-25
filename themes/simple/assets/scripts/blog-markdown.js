function init_code_copy_btn() {
    // 1. 遍历所有<pre>代码块容器，批量添加复制按钮
    document.querySelectorAll('.markdown pre>code').forEach(codeBlock => {
        // 判空：代码内容为空时不添加按钮
        if (!codeBlock.textContent) {
            return;
        }

        const preBlock = codeBlock.parentElement;

        // 跳过已有复制按钮的代码块，避免重复添加
        if (preBlock.querySelector('.code-copy-btn')) return;

        // 2. 创建复制按钮元素
        const copyImg = document.createElement('img');
        copyImg.src = "/theme/assets/image/copy.svg";

        const copyTxt = document.createElement('span');
        copyTxt.innerText = '复制';

        const copyBtn = document.createElement('button');
        copyBtn.className = 'code-copy-btn';

        copyBtn.appendChild(copyImg);
        copyBtn.appendChild(copyTxt);

        // 3. 将按钮添加到代码块容器中
        preBlock.appendChild(copyBtn);

        // 4. 绑定按钮点击事件：核心复制逻辑
        copyBtn.addEventListener('click', function () {
            // 5. 调用浏览器原生剪贴板API复制内容
            navigator.clipboard.writeText(codeBlock.textContent).then(() => {
                // 复制成功：修改按钮
                copyTxt.innerText = '复制成功';
                setTimeout(() => { copyTxt.innerText = '复制'; }, 1000);
            }).catch(err => {
                // 复制失败：捕获异常并提示（极少出现，一般是浏览器禁用剪贴板）
                tips_show("tips-item-error", `复制失败：${err}`);
                copyTxt.innerText = '复制失败';
                setTimeout(() => { copyTxt.innerText = '复制'; }, 1000);
            });
        });
    });
}