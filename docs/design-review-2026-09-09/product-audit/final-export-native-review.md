# v6 自动导出与仅补导出核验

本轮先实看 v6 原生截图及 AX，再只读检查实际导出文件、失败夹具基线、当前产物、任务记录和执行分支。没有操作 GUI、运行 cargo、触发转换或修改代码/夹具。完整计算结果在同目录 `final-export-byte-evidence.json`。

## 原生结果

- `v6-automatic-export-result.jpg/.ax.txt`：实际打开了《验收课程 · BV1UXPRODUCTEXPORT》，标题、来源、00:00、蓝色测试截图和人工字幕正文均可见；没有以成功状态表占据阅读正文。
- `v6-automatic-export-menu.jpg/.ax.txt`：导出菜单第一项是“打开导出文件夹”，与复制/新导出操作分组；不是要求用户再导出一次才能拿到自动生成的文件。
- `v6-automatic-export-finder-settled.jpg/.ax.txt`：Finder 已位于该笔记版本下的 `exports`，列表有 `course.html`。AX 的精确 URL 与下面检查的自动导出文件一致，因而不是只看到 Finder 被打开或停在版本父目录。
- `v6-onlyexports-library-reopened.jpg/.ax.txt`：重新打开的库中，《验收课程 · BV1UXPRODUCTDIRECT》有标题、来源、时长、截图数和阅读操作，没有“内容待补”或“部分完成”误导。下面的字节核验证实旧 manifest 仍保留注入的历史导出失败，这说明界面修正没有靠重写旧历史来清除提示。

这一批原生图可以关闭自动导出无可达入口、Finder 只落在版本目录，以及 optional exports 失败被标成正文缺失这三个具体问题。仅补导出文件的 Finder 点击属于主代理另一组证据，本批没有把自动导出的 Finder 图套用到它。

## 两份 HTML 都有正文和内联图片

使用标准 HTML 解析读取 body，检查每个 img 的 data URI，严格 base64 解码并重新计算 SHA-256。两份均为 3,341 bytes，含各自正确的课程标题、作者/时长、00:00 和正文“人工字幕：分散投资可以减少个别资产带来的影响。”，并含一个 `data:image/jpeg;base64` 图片。解码后都是 1,603 bytes，JPEG 文件头有效，SHA-256 与各自 `frames/slide_0001.jpg` 相同。没有用于呈现内容的外链 script/link/iframe/video/audio/source 资源；源视频超链接是用户可选的跳转，不是显示正文的依赖。

| 项目 | 实际文件 | HTML SHA-256 |
| --- | --- | --- |
| 普通任务自动导出 | `config-empty/course2md/desktop-local-library/course-7d00dd503ba2986dd63c414d6468928b/versions/task-18d3a4bde8a061d0-171ee-1/exports/course.html` | `846ad7fda3babc2dcb784ad7d975dbb74a01605032f985a4f3847ee17f2815cf` |
| 仅补导出 | `config-empty/course2md/desktop-local-library/course-cf5eb7305461ac3ce92992ac0490e670/exports/task-18d3a1419be8a868-cbfe-2/task-18d3a38fec3ca0d0-157f7-0/course.html` | `ff658b70cb0b8eff92dba879c5efd088f5c8bac18b03efde7644ce8211c52e95` |

路径均相对于本报告目录。两者使用同一张蓝色测试素材，所以图片字节相同；两个 HTML 的课程身份不同，整体哈希也不同。这是合成夹具，不能将纯蓝素材误判为缩略图丢失，也不能把此内容检查写成已在浏览器完成导出页面视觉验收。

## 正文、截图和旧版本没有被补做改写

仅补导出基线目录：`fixture-backup/onlyexports-BV1UXPRODUCTDIRECT`。`fixture-audit.json` 明确这是人工注入的 HTML 导出失败：注入前是已完成任务，注入后增加 HTML 请求/失败结果，用于从原生 UI 触发仅补导出。独立重算了 before/injected 文件哈希，均匹配该审计记录。

原版本 `task-18d3a1419be8a868-cbfe-2` 的五个内容文件，现在均与注入前审计哈希完全一致：

| 内容文件 | 注入前记录与当前共同的 SHA-256 |
| --- | --- |
| `timeline.jsonl` | `593e53ae3d46948fc237f25b684f528ed0a086f04739a7bf092a691f43a40d52` |
| `document.json` | `2eb17ff1fa3ced374ce24879b3013efc0018e7e65d76bd0f3df19ca28a88ec7a` |
| `course.md` | `8290408ab0d2955d6399aa1853a256efbd043c7654d92f417a1f34251235b5a0` |
| `meta.json` | `86d7c2a8add40950a2d434a8f5a59fe6d837b282029b8411916b19a5d0adf848` |
| `frames/slide_0001.jpg` | `b373d64501ddc0ebd4eb332e1160894990781a1c30a16cc7213ce8caff91d8df` |

当前 manifest 的 SHA-256 为 `1ebb461494130161b96cd33eb448ee60b4add1cdf1fb036b7423c67b70f48509`，逐字节等于 `manifest.injected.json`；仍保留原来成功的 transcript/screenshots、未请求的 proofreading/summary，以及注入的 html failed/partial 历史。`current.json` 也保持基线 SHA-256 `b741c70738441070612f9ac745a3720a53112bcbcc2afce0b2384966b998a74f`，仍指向原版本。该课程的 versions 目录只有这一个原版本。

新导出写到版本目录外独立的 `exports/<原版本>/<补做任务>/course.html`。新工作目录中的 `export-html.json` 收据哈希与实际 HTML 完全相同，`export-result.json` 的输出路径也一致。应用偏好基线中有记录的 `application.json` 哈希与默认库关联保持；没有把所有后来设置变化笼统说成未变化。

## 冻结计划与执行范围

计划使用 JSON 键排序、UTF-8、不转义 Unicode、紧凑分隔符后计算 canonical SHA-256；不是原 JSON 文件字节哈希。

| 计划 | Canonical SHA-256 |
| --- | --- |
| 人工注入前的旧任务 | `2bcf0c012eb463557ad8cdeb04ef4c9ecc548333c5d34d6a50507577036a501d` |
| 注入失败后的旧任务基线 | `0c17e1b77ea81b185aab2ce4c8823908e5a67c54a7879c57386e9eb55550e3a6` |
| 现在的旧任务 | `0c17e1b77ea81b185aab2ce4c8823908e5a67c54a7879c57386e9eb55550e3a6` |
| 新的仅补导出任务 | `be6b7ca2a29c5d527dc9483561a8b9d32c0589675b35eb708a8402d0f26fd724` |

注入前到注入后只改变计划中的 `config.defaults.formats: [] → [html]` 和 `options.formats: [false,false,false] → [false,true,false]`。这是制造“有导出请求但失败”的已记录夹具设置，**不能把 before 当补做启动基线而声称整个原始计划从未改变**。正确补做基线是 `desktop-workspace.injected.json`；它的所有既存任务计划现在均未改变。

新任务 `task-18d3a38fec3ca0d0-157f7-0` 的 parent 是原任务，旧任务的 handled_by 指向它。它与注入基线只在 operation 字段不同：`kind = reprocess`、`components = [exports]`，base_version_dir 指向原可读版本，prior_work_dir 指向原进度。来源、选项、服务/config 内容均继承该基线；HTML 是唯一输出格式，AI 校对/摘要关闭。

新任务为 complete，outcomes 只有 `exports.html = succeeded`，stages/logs 为空。新工作目录只有控制/身份/任务记录和两份导出记录，没有请求收据或新的正文、识别、截图产物；该原任务只有一个子任务。当前工作区还多了一份后来创建的 `BV1UXPRODUCTEXPORT` 普通自动导出任务，所以不能用“注入之后全工作区只新增一个任务”的旧总数断言替代按 parent 的核验。

旧工作目录的 task-record.json 现在与当前工作区旧任务一致。它**不等于注入前备份整文件**：差别包括已记录的 HTML 失败/格式注入、状态及后续 handled_by。与 injected 基线比较，只剩 handled_by/unread 的跟进状态变化，计划完全一致。新工作目录中的冻结计划也与新工作区任务计划一致。

最后只读核对 `src/pipeline.rs` 的 reprocess 分支：验证并读取旧版本后，`components` 全为 exports 时直接调用导出、写 export-result 并返回，位于生成新版本、AI 补做等分支之前。结合上述原文件哈希、单一旧版本、实际只含 exports 的冻结计划和收据，这条原生补做可判定为复用旧正文/图片进行导出，没有重跑正文生成。哈希相同本身只能证明没改字节，本结论没有只靠哈希推断执行过程。

本轮没有发现新的导出实质缺陷。关闭的是这组原生路径与合成失败夹具的行为；不扩大成所有格式、真实磁盘故障或所有导出页面视觉状态的通过结论。
