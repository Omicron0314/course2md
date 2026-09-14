# course2md 工程审查报告（v2.0.0-rc.3 基线）

- 日期：2026-09-14
- 范围：全仓库——`src/`（引擎+CLI）、`desktop/`（GPUI 桌面端）、`native/apple-asr`、构建/打包/CI 脚本。
- 方法：7 路并行只读审查（引擎核心、引擎支撑、desktop 三大页面、desktop 页面二组、desktop 页面三组、desktop 共享原语、native/构建），每路完整通读负责区域；另有主代理在 macOS 真机（M3 Max）上对 GPU 识别链路做了运行态复现。
- 分类约定：**[hack]** 权宜之计/硬编码/与文档不符；**[抽象不足]** 应共享未共享/魔法值散落；**[过度抽象]** 无谓分层/死参数化；**[工程化缺口]** 错误处理/取消/测试/IO 纪律缺失。
- 处置列：**待处置** → 本报告发布时的初始状态；随修复推进更新为 **已修复（commit）** 或 **后续建议**。

## 统计

| 区域 | 高 | 中 | 低 |
| --- | --- | --- | --- |
| 引擎核心（asr/pipeline/llm/fetch/dispatch/config） | 2 | 10 | 7 |
| 引擎支撑（其余 src/） | 0 | 7 | 13 |
| desktop 三大页面（reader/settings/task） | 3 | 11 | 8 |
| desktop 页面二组（import/workspace/onboarding/preferences） | 2 | 9 | 9 |
| desktop 页面三组（库/存储/诊断/导航） | 2 | 10 | 6 |
| desktop 共享原语与支撑层 | 2 | 8 | 5 |
| native/构建/打包/CI | 1 | 4 | 3 |
| 真机复现（主代理） | 1 | 1 | 0 |
| **合计** | **13** | **60** | **51** |

## 高严重度（13 + 1 复现）

| # | 分类 | 位置 | 问题 | 处置建议 | 处置 |
| --- | --- | --- | --- | --- | --- |
| H1 | 工程化缺口 | src/asr.rs:237（超时常量 asr.rs:29） | `wait_ready` 等 llama-server 就绪最长 300s，循环不读 control；桌面取消后 GPU 首次加载仍空等，子进程不被提前 kill | wait_ready 每轮 sleep 前读 control，非 run 即 kill 并返回取消 | 待处置 |
| H2 | 工程化缺口 | src/fetch.rs:618 + src/pipeline.rs:632-645 | yt-dlp 下载及重试循环全程不读 control，协作式取消在下载期间无效 | 每次 spawn 前 check_control；取消时 kill 当前 yt-dlp 且不再重试 | 待处置 |
| H3 | 工程化缺口 | desktop/src/reader_ui.rs:2414/1667-1768/3628-3673 | reader_page 渲染路径做 IO（canonicalize/read_dir/image_dimensions）、重建 ListState、清空查找框、每帧 Rc::default() | 加载/重置移到命令路径，render 只读快照 | 待处置 |
| H4 | 工程化缺口 | desktop/src/settings_ui.rs:786-788 | settings_page 每次绘制 hydrate 输入框并 spawn 模型诊断检查 | 进入设置/快照变化时执行，渲染无副作用 | 待处置 |
| H5 | 抽象不足 | desktop/src/reader_ui.rs:2396-4078 | reader_page 约 1680 行巨函数，空态/失败/完成不可单测 | 按区域拆纯函数（toolbar/meta/banner/article/gallery/toc） | 待处置 |
| H6 | 抽象不足 | import_ui.rs:2015 等 + main.rs:80-88 + workspace.rs:101 | 识别引擎三套并行编码（usize 索引 / &str id / AsrProvider 枚举），"5 即云端" 隐式契约散落 4 文件 | 集中索引↔枚举映射或直接持有 Option<AsrProvider> | 待处置 |
| H7 | 工程化缺口 | import_ui.rs:2623、onboarding.rs:1399/1986、views.rs:333/372/406 | new_page/setup_* 在渲染中启动模型文件系统检查并在渲染期改写 model_preparation 状态，违反 SKILL.md "渲染期不做 IO" | 检查触发移到事件入口；渲染期状态收敛移到事件/cx.defer | 待处置 |
| H8 | 工程化缺口 | desktop/src/main.rs:745-746 + course_library.rs:835-837 | 进入课程库即 refresh 且 loading 时整页替换为「正在读取笔记…」，已在内存的列表闪空 | 刷新期间保留旧列表 + 非破坏性指示 | 待处置 |
| H9 | hack | course_library.rs:2062-2065、main.rs:1278-1281 | 「查看任务」/打开笔记成功直接写 this.page，绕过 navigate() 的草稿保存/阅读位置/跟随清理 | 全部走 navigate()（或抽 leave_page） | 待处置 |
| H10 | hack | desktop/src/a11y.rs:8-39 | 文档承诺 panic payload（含用户数据）omitted，实现却把 payload 原文写日志——文档与实现矛盾且违背隐私承诺 | 删除 payload 记录，只留 location+backtrace | 待处置 |
| H11 | hack | desktop/src/views.rs:598-646（同 import_ui.rs:3021-3026 等） | 测试 include_str! 读自身源码做字符串断言，测源文本而非行为 | 删除源码嗅探测试，改行为/布局断言 | 待处置 |
| H12 | 工程化缺口 | build.rs:13-14 | rerun-if-changed 漏 Package.resolved，swift 依赖更新后本地构建静默链旧静态库 | 补一行 rerun-if-changed | 待处置 |
| H13 | 工程化缺口 | 真机复现：settings 引擎选择 + choice_group | macOS 安装版（2.0.0）中选择 GPU 不持久：点击仅聚焦不提交，随后无关键盘/滚动事件把选择静默改为 coreml 并落盘（generation.json rev27→28 实证）；用户视角即「不识别 GPU/选了没用」。HEAD 行为待复测 | 在 HEAD 复现定位；修复提交/回退链路并加回归测试 | 待处置 |
| H14 | hack | src/asr.rs:746 parse_gpu_devices | doctor 把 `BLAS: Accelerate`（CPU 后端行）列为 GPU 设备——任何 name: desc 行都被收，不过滤 CPU-only 设备 | 过滤已知非 GPU 前缀（BLAS/CPU 等），与 desktop backend.rs 的 MTL/CUDA/Vulkan/SYCL/ROCm 白名单对齐为共享判定 | 待处置 |

## 中严重度（60 条，按区域）

### 引擎核心（10）
- [工程化缺口] src/asr.rs:951/1234/324-349 — ffmpeg_vad/cut_wav 无超时、非 ManagedChild、未 stdin(null)，ffmpeg 挂死会卡住取消路径。处置：对齐 ManagedChild + stdin null + 超时。｜待处置
- [工程化缺口] src/pipeline.rs:346-360 + fetch.rs:618-620 — 补做截图路径 download 见文件即 Ok 并 save_file_digest，把未校验文件标为已校验（主路径 prepare_cached_media 防过的洞）。处置：补做走 prepare_cached_media 或 download 先校验 marker。｜待处置
- [工程化缺口] src/asr.rs:945-948 — 本地 transcribe_file 空文本 bail 致整次 ASR 失败；云端同情形当静音完成。处置：空文本返回 Ok("") 对齐语义。｜待处置
- [hack] src/config.rs:491-500 — resume 注释「默认关闭」与实现 `unwrap_or(true)`、测试断言三者矛盾。处置：二选一改准。｜待处置
- [hack] src/llm.rs:732-735 — test_connection 空 key 也发 `Bearer ` 头，且不走 dispatch::receive（默认跟随重定向，违反账本 redirects(0) 约束）。处置：复用生产 header/agent 规则。｜待处置
- [抽象不足] src/fetch.rs:624-669 vs 707-735 — 下载对任意错误（含 401/404）重试 3 次，探测只重试 412；确定性错误被放大。处置：仅瞬时错误/412 重试并共用 bilibili_retry_delay。｜待处置
- [工程化缺口] src/pipeline.rs:651 + asr.rs:72-77/247 — 截图与转写 join! 一路失败不取消另一路；阻塞 HTTP 120-300s 只在请求前 check_control。处置：select 取消 + 更短超时/可中断 client。｜待处置
- [工程化缺口] src/llm.rs:673-682/191-210 — 重试 backoff sleep 不看 control；vision 读图+base64 无取消点。处置：退避拆短睡并轮询 control。｜待处置
- [工程化缺口] src/dispatch.rs:416 — 每次 send 前 read_dir 全量解析收据，N 段云端 ASR 为 O(N²) 磁盘扫描。处置：Ledger 内维护索引。｜待处置
- [hack] src/config.rs:196/589 vs pipeline.rs:53 — out_root 注释/course_dir() 描述的目录布局与实际 `out_root/{platform}/{course_id}` 不符。处置：改注释与测试。｜待处置

### 引擎支撑（7）
- [抽象不足] doctor.rs:60-77 vs apple.rs:117-138 — metallib 检测位置清单两处不一致，结论可互相矛盾。处置：共享探测函数。｜待处置
- [hack] models.rs:24-37 — normalize_apple_model 用 contains("0.6")/contains("whisper") 猜模型名，未知输入被静默误映射。处置：精确别名表 + 未知名报错。｜待处置
- [工程化缺口] artifact.rs:290-380 — publish 是 async fn 却全程阻塞 std::fs（复制/SHA-256/fsync），与 spawn_blocking 纪律不一致。处置：重 IO 移 spawn_blocking。｜待处置
- [抽象不足] config.rs:558/doctor.rs:96/wizard.rs:168 — NPU 设备路径硬编码三处；doctor 仅 linux 检查 NPU，Windows 不报。处置：共享 NPU 检测并统一平台门槛。｜待处置
- [hack] main.rs:50-53 — clap 解析前手工扫 argv 找 --json/run-task，参数值含 "--json" 会误切 NDJSON。处置：注释声明近似或精确判断。｜待处置
- [抽象不足] wizard.rs:237/245 vs settings.rs:72-74/216-219 — 云端默认 base_url/model 字面量写三份。处置：从 AsrApi::default() 读取。｜待处置
- [工程化缺口] models.rs:345 — 2.4GB 模型下载失败留 .part 但永不续传。处置：Range 断点续传或失败后清理 .part。｜待处置

### desktop 三大页面（11）
- [hack] settings_ui.rs:193-218/372-381 — setting_label/settings_detail_group 用中文标签子串猜图标。处置：调用点显式传 Icon。｜待处置
- [hack] reader_ui.rs:2912-2915/4113-4143 — 目录选中态手写 ACCENT_SOFT 覆盖共享控件。处置：抽 selected_row/toggle_chip 或复用 SingleChoiceGroup surface。｜待处置
- [抽象不足] task_ui.rs:2251-2678/3048-3184/3186-3393 — 暂停/取消/继续/不确定结果/修复 AI 等动作块在队列卡与工作台各写一遍，文案已漂移。处置：抽 TaskActions/task_recovery_block。｜待处置
- [工程化缺口] task_ui.rs:1414-1472 — start_next_task 先标 Running 再启动进程，启动失败且补偿失败会留下永不调度的 Running；补偿错误被 let _ 吞。处置：先启动后提交或同事务补偿+硬提示。｜待处置
- [工程化缺口] task_ui.rs:1963-1976 + settings_ui.rs:4098-4101 — finish_task/添加课程库在 UI 线程做 cover 保存/canonicalize/read_to_string。处置：移 background_executor。｜待处置
- [hack] settings_ui.rs:1250/3936 — Textarea 固定 px 高度多行面，不随字号/窗口回流（违反设计合同）。处置：按 rem/内容高度。｜待处置
- [抽象不足] settings_ui.rs:1556-1566/1865-1876 — 「每 service 最新 version」BTreeMap 折叠复制两处。处置：抽 latest_versions。｜待处置
- [工程化缺口] reader_ui.rs:1902 — 重新定位原视频的 probe 取消标志永不置位。处置：AtomicBool 入 reader_ui 并在关闭/切换时置位。｜待处置
- [工程化缺口] reader_ui.rs:1344/3898/4581 — 图片 fallback 共用 ElementId "failed-reader-image"，多图失败 ID 碰撞。处置：带 index/哈希。｜待处置
- [工程化缺口] task_ui.rs:2028-2095 — queue_page 每帧 clone 全部 TaskRecord，绘制时写 entered。处置：Rc 快照 + 渲染前计算。｜待处置
- [抽象不足] settings_ui.rs:2422-3059 — service_editor_content 约 640 行单函数。处置：拆 connection/auth/test/footer。｜待处置

### desktop 页面二组（9）
- [hack] import_ui.rs:3021-3026/3754-3786/3830-3855 — include_str! 自身源码断言。处置：删除，改行为测试。｜待处置
- [过度抽象] import_ui.rs:74-91/107-109/2111-2113/2735 — 三个恒值「配置」函数拖死分支（unreachable!()）。处置：删函数与死分支，设计决定写注释。｜待处置
- [抽象不足] import_ui.rs:616-619 vs 1333-1336 — 错误摘要分类（WARNING:/ERROR: 子串）两处重复。处置：抽 summarize_source_error。｜待处置
- [抽象不足] import_ui.rs:668-761 vs 832-934 — 两个字幕作业的大段取消/代际守卫样板复制。处置：抽共享作业原语。｜待处置
- [hack] onboarding.rs:438-456 — model_preparation_phase 靠进度消息子串嗅探推断阶段，引擎改文案即退化。处置：结构化阶段事件或集中关键词常量。｜待处置
- [抽象不足] onboarding.rs:184 等 9 处 + 5 文件 — 默认模型 "qwen3-1.7b" 硬编码散落，显示名两个版本。处置：共享常量 + 单一显示名映射。｜待处置
- [抽象不足] main.rs:80-88/import_ui.rs:2184-2192/onboarding.rs:208-217/model_diagnostics.rs:73-81 — 同一 AsrProvider 的展示标签四个出处且措辞不一。处置：单一 provider_label()。｜待处置
- [抽象不足] preferences.rs:928-955 vs 956-983 — save_generation/save_application 的 intent 恢复逻辑逐字重复两遍。处置：按 PreferenceGroup 参数化泛型助手。｜待处置
- [抽象不足] workspace.rs:353-355/1140-1142 — course-{digest[..32]} 目录名重复三处，32 截断无注释。处置：抽 course_dir_name()。｜待处置

### desktop 页面三组（10）
- [工程化缺口] notes.rs:21-32/45-54 — Course::from_completed 吞 manifest 读失败，storage_dir 回退逻辑致文件夹归属丢失。处置：要求可读 manifest 或显式 course 根。｜待处置
- [hack] course_library.rs:390-392 + main.rs:1279 — apply_course_title_aliases 追加不清空，诊断重复累积。处置：覆盖/去重。｜待处置
- [抽象不足] main.rs:1040-1045 + model_diagnostics.rs:657-670 — 模型下载进度与任务进度路径分裂：诊断面板 1s 合帧 vs 任务 250ms；stage 过滤用 starts_with/contains 字符串启发式；两处文案不一。处置：抽 is_model_transfer_stage，统一合帧与文案。**（信息展示重点）**｜待处置
- [抽象不足] model_diagnostics.rs:231-237/250-278/793-798 — preparing/result 全局一份且完成后不清 preparing，旧 notice 可错位。处置：结果按 key 存放，结束清 preparing。｜待处置
- [抽象不足] model_diagnostics.rs:64-72 vs activity.rs:278-287 — 两套 bytes()，<1MB 显示「512000 字节」。处置：共用 activity 格式化。｜待处置
- [工程化缺口] credentials.rs:235 — 非 macOS 凭据 load 失败 unwrap_or_default 当空表，后续 insert 覆盖全文件丢密钥。处置：损坏拒绝写入并报错。｜待处置
- [工程化缺口] storage.rs:315-333 — pending_journals 读失败静默跳过，半残 journal 不可续传/放弃。处置：损坏 journal 也进入 UI。｜待处置
- [工程化缺口] library_ui.rs:147-156 — 源探测回调绑 cx.windows().first()，窗口已关则 preview_workers 永不递减，request_close 拖到 10s。处置：捕获发起窗口 handle，保证递减。｜待处置
- [抽象不足] main.rs:203-204/library_ui.rs:208-210/347-351 — 未分类三义（Some(0)/None/记录缺席）；save_folder 直接改 page 不走 navigate。处置：枚举化 + 走 navigate。｜待处置
- [工程化缺口] storage.rs:278-281 — sync_all 仅 Unix，Windows 空操作但状态仍标 Verified/Committed。处置：Windows FlushFileBuffers 或文档明示。｜待处置

### desktop 共享原语（8）
- [过度抽象] theme.rs:412-425/653-678/57-58 + choice_group.rs:107/120 — reveal/disclosure/banner_note/SIDEBAR/COVER/disabled builder 全仓零调用方。处置：rc 前死代码清扫。｜待处置
- [抽象不足] theme.rs:719/settings_ui.rs:4155/preferences.rs:601 — 字号档位 [1.0,1.25,1.5,2.0] 三处硬编码。处置：共享 FONT_SCALES 常量。｜待处置
- [抽象不足] views.rs:118/351/theme.rs:243 等 — 标题栏几何与面板宽度魔法数多处重复。处置：TITLE_BAR_HEIGHT/NAV_WIDTH 命名常量。｜待处置
- [抽象不足] service_test.rs:108-154 vs model_discovery.rs:281-310 — 两套单实现 Transport + 近乎逐行复制的 ureq bounded 管道，MAX_RESPONSE 同名不同值。处置：抽共享 bounded-request 帮助函数。｜待处置
- [抽象不足] theme.rs:132-140 — apply_preference 内联 ease_out 同式与 280ms 不复用 motion 词汇表。处置：复用 motion::ease_out + PALETTE_MS 常量。**（动画相关）**｜待处置
- [hack] backend.rs:81-95 — 脱敏用硬编码 JSON 路径抠引擎 Request 密钥，引擎改字段名则密钥静默进日志。处置：引擎导出敏感字段清单/secrets() 访问器。｜待处置
- [抽象不足] views.rs:584/main.rs:1524/import_ui.rs:2807/settings_ui.rs 多处 — settings_tab 魔法下标散落四文件且有 ==5 越界钳制残留。处置：SettingsTab 枚举。｜待处置
- [工程化缺口] backend.rs:306-360 — Environment::detect 用位置下标回填字段，重排即静默错配。处置：(name,bin,arg) 与字段赋值写在一起。**（GPU 检测相关）**｜待处置

### native/构建/CI（4）
- [hack] justfile:37/package.py:91/homebrew 模板/Package.swift:6 — 最低系统版本三处口径不一（14.0 vs sequoia vs macOS v15）。处置：统一权威值。｜待处置
- [工程化缺口] package.py:134-135/148 + release.yml:182-189 — 有签名身份但公证材料不全时静默跳过公证，产出被 Gatekeeper 拦截的 DMG 且无告警。处置：显式报错或醒目 warning。｜待处置
- [抽象不足] desktop.yml:15-45 vs release.yml:128-200 — 桌面打包流水线复制粘贴且已漂移（ffmpeg 依赖有无、test 命令不一致）。处置：composite action/workflow_call。｜待处置
- [工程化缺口] shim.swift:128-152 — 手工镜像上游 HF 仓库名与文件清单，上游改布局则缓存准备成功而加载失败。处置：CI 加「prepare 后离线加载」冒烟。｜待处置

## 低严重度（51 条）

引擎核心 7：asr.rs:1135 Energy::load 吞错；llm.rs:198-201 无图 chunk 不 inc 进度条停不满；config.rs:466 tilde 展开 Windows 不认 USERPROFILE；asr.rs:34 云端 ASR 并发写死 4；pipeline.rs:47 probe_duration 失败静默当 0；dispatch.rs:171-178 Guard Drop 锁中毒后不再清；llm.rs:615-619 is_retryable 仅 cfg(test) 死路径。

引擎支撑 13：doctor/wizard provider 标签两份 drift；默认模型字面量分散；pick_subtitle_file 只认小写 .srt；两个文件名净化器规则各异；failure_message 按 " / " 截双语串；render.rs write_outputs 不走 atomic_write；apple.rs vad() 不校验 FFI 空指针；models.rs prepare 吞二次检查错误；#[path] 非常规模块布局；wizard.rs:128 中途 exit(0)；NPU worker HTTP 无鉴权 token；models.rs Api 分支两种防御风格并存；progress.rs Mutex unwrap 毒化 panic。

desktop 三大页面 8：settings_tab==5 渲染期钳制残留；provider==5 魔法数与默认模型字面量；reader_image_actions 恒等 reveal 闭包；ordinary_preferences_ready_for_submit 等薄包装/空实现；task_component_failures 死参数；正文插图有真实尺寸却写死 16/9；request_layout expect；service_configuration_issue 子串分类。

desktop 页面二组 9：advance_conversion_when_ready 抓第一个窗口；box_section 中文案 match 图标；stop_session 吞 set_intent 错误；public_config 脱敏清单无编译期保障；apply_* 一行 setter 主要为测试存在；onboarding 死参数/兜底分支/Debug 格式 ID；步骤元数据两处真相；existing_source_note 每渲染遍历克隆全部课程；onboarding 面板宽度公式与断点魔法数重复。

desktop 页面三组 6：folder_editor 死删除确认块；Drop 不取消 subtitle_cancel；Keychain/Keyring 删除语义不一致；source.rs probe 语言预排序与 UI 不一致；notes.rs manifest unwrap + 扫描深度魔法 8；model_diagnostics 写死 qwen3-1.7b 两处。

desktop 共享原语 5：icons 资产表与函数清单手工双份 + 双名别名混用；youtube 品牌色值与注释不符；motion enter/state_enter 死参数 _cx；spawn_input UI 线程同步写 stdin 契约无文档；性能开关环境变量解析不一致。

native/构建 3：bench-mac.sh sudo pkill powermetrics 误杀 + 吞被测失败；install.sh 无校验和验证；install.sh mlx.metallib 下载失败被吞。

## 总体观察（跨区域共识）

1. **取消契约未铺满是最集中的工程化缺口**：引擎的最长操作（llama-server 加载 300s、yt-dlp 下载、ffmpeg 切段、阻塞 HTTP 120-300s）几乎不轮询 control.json；desktop 侧也有不可取消的 probe/渲染期 IO。直接影响「任务暂停/取消」可信度与信息展示。
2. **渲染期副作用是 desktop 的系统性违规**（H3/H4/H7 + 多处渲染期状态改写），与 SKILL.md 明文约束冲突，也是设置页状态混乱（含 H13 选择回退）的温床。
3. **同一事实多份拷贝已开始漂移**：provider 编码/标签（4 处）、默认模型（9 处）、metallib 路径、NPU 路径、字号档位、标题栏几何、HTTP 管道、密钥脱敏路径。建议设立「后端探测与默认值」单一事实源模块。
4. **持久化/崩溃恢复的主路径质量很高**（checkpoint/artifact/preferences/workspace 均有行为级测试），但补做下载校验、render 输出、凭据损坏、journal 损坏等「第二路径」没获得同等待遇。
5. **测试文化两元化**：支撑层（choice_group/motion/backend/dispatch）有真实行为测试；同时存在 include_str! 源码哨兵测试与恒值函数+死分支的「设计钉桩」机制，后者应整体清除。
6. **没有架构性过度抽象**：单实现 trait 均服务于测试注入，合理；过度抽象集中在死 API/薄包装/恒值函数层面，适合一次清扫。
7. **信息展示的直接证据**：模型下载进度两处消费者+字符串过滤+1s 合帧（设置页明显滞后）、进度条 total 含跳过 chunk 停不满、字节格式化两套、补做路径把未校验文件标已校验——与用户感知的「进度条让人焦虑」吻合。

## 真机复现记录（macOS M3 Max，2026-09-14）

- CLI `course2md doctor`（2.0.0-rc.2 二进制）：正确识别 `MTL0: Apple M3 Max`，但同时把 `BLAS: Accelerate` 列为 GPU（H14）。
- 安装版 2.0.0（/Applications/course2md.app，bundle dev.course2md.desktop，pid 7302，cua-driver 截图取证）：
  - 设置→生成笔记→高级设置→本机引擎：GPU 选项可见可聚焦。
  - 点击 GPU：出现聚焦环，识别模型行短暂变为单选宽版，但 `generation.json` 从未写入 provider="gpu"（实证：rev 27 null → rev 28 coreml）。
  - 随后一次 cua-driver scroll（注入 PageDown/方向键）把已聚焦的选择组导航到 coreml 并提交落盘——用户视角为「选 GPU 没反应，过一会儿自己跳回 Apple 原生」。
  - 结论：「不识别 GPU」的表层症状 = GPU 可检测（doctor 证明）但**桌面端选择不持久 + 静默回退**；根因方向：设置渲染期的状态水合/提交链路（H4/H13 区域）与 choice_group 键盘导航提交语义。HEAD 复测后定位到具体代码。
