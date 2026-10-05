# GPUI 组件库完整组件清单

> 覆盖场景：代码编辑器 / IDE、文档与知识库、股票与金融平台、AI 桌面、AI 对话平台、设计工具、数据库与开发工具、IM 与邮件、项目管理、媒体、系统工具等桌面应用。

---

## 1. 基础原语（Primitives）

- Box / View — 最底层容器
- Text — 纯文本渲染
- Icon — 矢量图标
- IconSet / IconRegistry — 图标集注册与按名称加载
- Svg — 任意 SVG 渲染
- Image — 图片（加载中、失败、占位）
- Canvas — 自定义绘制区域
- Divider / Separator — 水平 / 垂直分割线
- Spacer — 弹性占位
- Portal — 渲染到顶层图层
- Overlay / Backdrop — 遮罩层
- Slot — 插槽 / 组件组合
- Show / When — 条件渲染
- For / Each — 列表渲染辅助
- Fragment — 无包裹分组
- VisuallyHidden — 仅屏幕阅读器可见
- FocusRing — 焦点环
- FocusScope / FocusTrap — 焦点作用域与限制
- ClickOutside — 外部点击检测
- HoverArea — 悬停检测区域
- Pressable — 可按压区域（按下态、长按）
- KeyboardHandler — 键盘事件区域
- Measure — 尺寸测量
- ResizeObserver — 尺寸变化监听
- IntersectionObserver — 可见性监听
- Clipboard — 剪贴板读写
- Tooltip Trigger — 通用提示触发器

---

## 2. 排版（Typography）

- Heading（H1–H6）
- Title / Subtitle
- Paragraph
- Label
- Caption
- Overline
- Code（行内代码）
- Kbd — 快捷键显示（支持 ⌘ / Ctrl 平台差异）
- KbdCombo — 组合键显示（⌘ + Shift + P）
- Blockquote
- Highlight / Mark — 高亮文本（搜索匹配）
- Truncate — 单行截断
- LineClamp — 多行截断
- MiddleEllipsis — 中间省略（长文件路径）
- EllipsisTooltip — 截断后悬停显示全文
- Link — 链接
- ExternalLink — 外部链接（带图标）
- RelativeTime — 相对时间（3 分钟前）
- DateTimeText — 格式化时间
- NumberText — 数字格式化（千分位、精度）
- CurrencyText — 货币显示
- PercentText — 百分比显示
- FileSizeText — 文件大小（KB / MB / GB）
- DurationText — 时长（01:23:45）
- PluralText — 复数处理
- CopyableText — 可复制文本
- SelectableText — 可选中文本
- Emoji — 表情渲染
- Latex / MathInline — 行内公式
- AnimatedNumber — 数字滚动动画
- Typewriter — 打字机效果文本
- GradientText — 渐变文字
- ShimmerText — 闪光文字（加载中）

---

## 3. 布局（Layout）

- Stack / HStack / VStack
- Flex
- Grid / GridItem
- SimpleGrid — 等宽响应网格
- Masonry — 瀑布流
- Center
- Container — 最大宽度约束容器
- AspectRatio
- Inset / Padding 容器
- Wrap — 自动换行布局
- Absolute / Positioned — 绝对定位容器
- Layer / ZStack — 层叠
- ScrollArea — 自定义滚动区域
- Scrollbar — 可样式化滚动条（悬停显示、覆盖式）
- ScrollShadow — 滚动边缘阴影
- ScrollToTop — 回到顶部
- StickyHeader — 吸顶
- SplitPane — 可拖拽分栏（水平 / 垂直 / 嵌套）
- ResizablePanel / ResizablePanelGroup
- ResizeHandle — 拖拽手柄
- Collapsible — 可折叠区域
- Accordion / AccordionItem
- Card / CardHeader / CardBody / CardFooter
- Panel / PanelHeader
- Section / SectionHeader
- Group / Fieldset
- Frame — 带边框区域
- Well — 内嵌凹陷区域
- Sidebar — 可折叠侧边栏
- Drawer — 抽屉
- Sheet — 边缘滑出面板
- Page / PageHeader / PageContent
- AppShell — 应用骨架（标题栏 + 侧栏 + 内容 + 状态栏）
- MasterDetail — 主从布局（列表 + 详情）
- ThreeColumnLayout — 三栏布局（邮件 / IM）
- Dock / DockPanel — 可停靠面板系统（上下左右停靠、浮动、合并为标签组）
- DockZone / DropIndicator — 停靠拖拽指示区域
- FloatingPanel — 浮动面板
- Pane / PaneGroup — 编辑器分屏
- Workspace — 工作区布局持久化
- Viewport — 画布视口（缩放平移）

---

## 4. 窗口与桌面外壳（Window & Shell）

- TitleBar — 自定义标题栏（可拖动区域）
- ColumnShell — 分栏外壳：侧栏通到窗口顶部，每栏自带可拖动标题（双栏 / 三栏）
- WindowControls — 最小化 / 最大化 / 关闭（macOS 红绿灯、Windows、Linux 样式）
- TrafficLights — macOS 风格窗口按钮
- WindowDragRegion — 窗口拖拽区域
- WindowResizeBorder — 无边框窗口缩放边缘
- MenuBar — 应用菜单栏（文件 / 编辑 / 视图…）
- NativeMenu 适配层
- Toolbar / ToolbarGroup / ToolbarSeparator
- StatusBar / StatusBarItem — 底部状态栏
- ActivityBar — 侧边图标导航（VS Code 风格）
- NavigationRail
- TabBar（窗口级）
- SplashScreen — 启动页
- AboutDialog — 关于窗口
- TrayIcon / TrayMenu — 系统托盘
- DockBadge — Dock / 任务栏角标
- JumpList — 任务栏快捷菜单
- SystemNotification — 系统级通知桥接
- MultiWindow Manager — 多窗口管理
- WindowSwitcher — 窗口切换器
- QuickLauncher — 快速启动器（类 Spotlight / Raycast）
- MiniWindow / CompactMode — 迷你模式窗口
- AlwaysOnTop 浮窗
- PictureInPicture — 画中画窗口
- UpdateBanner / UpdateDialog — 应用更新提示
- CrashReporter — 崩溃报告界面
- OfflineIndicator — 离线状态
- ZoomControl — 界面缩放

---

## 5. 按钮与操作（Buttons & Actions）

- Button（primary / secondary / outline / ghost / subtle / danger / link）
- IconButton
- ButtonGroup
- SplitButton — 主按钮 + 下拉
- DropdownButton / MenuButton
- ToggleButton
- ToggleGroup
- SegmentedControl
- FloatingActionButton
- LoadingButton — 带加载态
- ConfirmButton — 二次确认按钮（按住确认 / 点击两次）
- HoldToConfirm — 长按确认
- CopyButton — 复制并反馈
- ShareButton
- CloseButton
- BackButton
- MoreButton（…）
- ActionBar — 底部 / 浮动操作栏
- BulkActionBar — 批量选择后操作栏
- ActionSheet
- QuickActions — 快捷动作列表
- ShortcutHint — 按钮上附带快捷键提示

---

## 6. 表单输入（Form Inputs）

### 文本类
- Input / TextField
- TextArea（自动增高、最大行数）
- PasswordInput（显示 / 隐藏）
- SearchInput（清除、搜索历史）
- NumberInput（步进、键盘上下、拖拽改值）
- ScrubInput — 拖拽改数值（设计工具风格）
- CurrencyInput
- PercentInput
- UnitInput — 带单位输入（px / % / em）
- EmailInput
- UrlInput
- PhoneInput（区号）
- MaskedInput — 格式掩码
- PinInput / OTPInput
- InlineEdit / EditableText — 点击即编辑
- InputGroup / InputAddon — 前后缀
- ClearableInput
- AutoResizeInput
- MentionInput — @ 提及
- HashtagInput
- TagInput / TokenInput
- KeyValueInput — 键值对编辑
- ListInput — 可增删的字符串列表
- PathInput — 文件路径输入 + 浏览
- RegexInput — 正则输入（带校验高亮）
- ExpressionInput — 公式 / 表达式输入
- HotkeyInput / ShortcutRecorder — 快捷键录制

### 选择类
- Checkbox（含半选态）
- CheckboxGroup
- CheckboxCard
- Radio / RadioGroup
- RadioCard
- Switch / Toggle
- Select
- NativeSelect
- Combobox — 可搜索下拉
- Autocomplete
- MultiSelect
- CascadeSelect / Cascader — 级联选择
- TreeSelect — 树形选择
- TransferList — 穿梭框
- ListBox — 列表选择
- ChoiceChips / FilterChips
- Rating
- Slider
- RangeSlider
- VerticalSlider
- Knob / Dial — 旋钮（音频类）
- Stepper（数值加减）

### 日期时间
- Calendar
- DatePicker
- DateRangePicker
- TimePicker
- DateTimePicker
- MonthPicker / YearPicker / QuarterPicker
- WeekPicker
- RelativeDatePicker（最近 7 天、本月…）
- TimezoneSelect
- DurationPicker
- CronEditor — 定时规则编辑

### 颜色
- ColorPicker（HSV / RGB / HEX / Alpha）
- ColorSwatch
- ColorPalette
- EyeDropper — 取色器
- GradientEditor

### 文件
- FilePicker / FileInput
- FolderPicker
- DropZone — 拖拽上传区域
- UploadList — 上传进度列表
- ImageUpload（带预览、裁剪）
- AvatarUpload

### 其他
- SignaturePad — 签名板
- CodeInput — 单行代码输入（带高亮）
- JsonInput — JSON 输入（校验）
- FontPicker
- IconPicker
- EmojiPicker
- LanguageSelect
- CountrySelect
- CurrencySelect

### 表单结构
- Form
- FormField
- FormLabel（必填标记、帮助图标）
- FormDescription / HelperText
- FormError / ValidationMessage
- FormSection
- FormActions
- FieldArray — 动态字段组
- InlineForm
- WizardForm / MultiStepForm
- DirtyIndicator — 未保存改动提示

---

## 7. 导航（Navigation）

- Tabs（顶部 / 左侧 / 底部）
- EditorTabs — 可关闭、可拖拽排序、可固定、预览标签（斜体）、未保存圆点、溢出菜单
- TabOverflowMenu
- Breadcrumb（可点击每级展开同级下拉）
- Pagination
- LoadMore
- Stepper / Steps / Wizard
- Sidebar Navigation / NavItem / NavGroup
- NavigationMenu
- ActivityBar Item（带角标）
- BackForwardNavigation — 前进后退
- History Navigation
- Anchor / TableOfContents 导航
- JumpTo / GoToLine
- CommandPalette — 命令面板（分组、最近使用、模糊匹配高亮）
- QuickOpen — 快速打开文件
- QuickSwitcher — 快速切换（文档 / 会话 / 项目）
- SearchPalette — 全局搜索
- SpotlightSearch
- Tour / Onboarding 导航点

---

## 8. 菜单（Menus）

- Menu / MenuItem / MenuGroup / MenuSeparator
- SubMenu — 级联子菜单
- CheckboxMenuItem
- RadioMenuItem
- MenuItemShortcut — 右侧快捷键
- MenuItemIcon
- ContextMenu — 右键菜单
- DropdownMenu
- MenuBar Menu
- OverflowMenu（…）
- MegaMenu
- SearchableMenu — 带搜索框的菜单
- PieMenu / RadialMenu — 环形菜单

---

## 9. 浮层（Overlays）

- Tooltip（延迟、跟随鼠标、富内容）
- Popover
- HoverCard — 悬停卡片（用户信息、链接预览）
- Dialog / Modal
- AlertDialog
- ConfirmDialog
- PromptDialog — 输入对话框
- FullscreenDialog
- Sheet
- Drawer
- Lightbox
- Spotlight / Coachmark — 聚焦引导
- FloatingToolbar — 选中后浮动工具栏
- Callout / Peek — 内联浮窗（类 Peek Definition）
- DropdownPanel
- InlinePopup
- ToastViewport — 通知容器

---

## 10. 反馈（Feedback）

- Toast（成功 / 失败 / 警告 / 信息 / 带操作 / 可撤销）
- Notification — 带标题、正文、操作
- NotificationCenter — 通知中心列表
- Alert（inline）
- Banner — 顶部横幅
- Callout / Admonition（提示、注意、危险）
- InlineMessage
- StatusMessage
- Snackbar
- UndoToast — 撤销提示
- ErrorView — 错误页
- ErrorBoundary
- NotFound / 404
- NoPermission / 403
- Maintenance — 维护中
- Offline — 离线
- EmptyState（无数据 / 无搜索结果 / 首次使用）
- Result — 结果页（成功 / 失败）
- ConfirmationCard
- SavingIndicator — 保存中 / 已保存
- SyncStatus — 同步状态
- ConnectionStatus — 连接状态
- Countdown — 倒计时
- Timer

---

## 11. 加载与动画（Loading & Motion）

### 加载
- Spinner（圆环、点阵、条形、脉冲等多种样式）
- DotsLoader — 三点跳动
- BarsLoader
- PulseLoader
- RingLoader
- OrbitLoader
- WaveLoader
- Skeleton / SkeletonText / SkeletonAvatar / SkeletonCard / SkeletonTable
- Shimmer — 流光效果
- ProgressBar（确定 / 不确定 / 分段 / 缓冲）
- ProgressCircle / ProgressRing
- StepProgress
- LoadingOverlay — 覆盖式加载
- LoadingScreen — 全屏加载
- InlineLoader
- LazyLoad — 延迟加载容器
- Suspense / AsyncView — 异步加载状态封装
- RefreshIndicator
- TypingIndicator — 正在输入

### 动画与过渡
- Transition（fade / slide / scale / collapse）
- AnimatePresence — 进出场动画
- Motion — 声明式动画
- Spring — 弹簧动画
- Stagger — 交错动画
- Reorder — 列表重排动画
- Flip — 布局变化动画
- CountUp — 数字递增
- Marquee — 跑马灯
- Ripple — 点击波纹
- Confetti — 彩带庆祝
- Glow / Pulse — 发光 / 呼吸
- Blink / Flash — 闪烁（价格变化）
- Shake — 抖动（校验失败）
- ParticleBackground
- AnimatedGradient
- Lottie Player

---

## 12. 数据展示（Data Display）

- Badge / CountBadge / DotBadge
- Tag / Chip（可关闭、可选择）
- Avatar（图片 / 首字母 / 图标 / 在线状态）
- AvatarGroup — 堆叠头像
- UserChip — 头像 + 名字
- Statistic / KPI Card
- Metric — 指标 + 趋势
- TrendIndicator — 上涨 / 下跌箭头
- DescriptionList
- PropertyGrid — 属性面板（IDE / 设计工具）
- KeyValueList
- InfoRow
- Timeline / TimelineItem
- ActivityFeed — 动态流
- Changelog
- Carousel
- Gallery
- Accordion 列表
- Card Grid
- QRCode
- Barcode
- ColorSwatch 展示
- Meter — 容量条（磁盘、配额）
- Gauge
- UsageBar — 分段用量条
- Rating 展示
- Comparison — 对比视图
- BeforeAfter — 前后对比滑块
- Watermark

---

## 13. 列表与树（Lists & Trees）

- List / ListItem（主文本、副文本、前后缀、操作）
- VirtualList — 虚拟滚动（可变高度）
- InfiniteList — 无限滚动
- SortableList — 拖拽排序
- SelectableList — 单选 / 多选 / Shift 范围选
- GroupedList — 分组列表 + 分组吸顶
- SectionList
- ReorderableList
- SwipeableListItem
- Tree / TreeView（展开、多选、拖拽、懒加载、重命名、键盘导航）
- VirtualTree — 大数据量树
- FileTree — 文件树（图标、Git 状态、过滤）
- CheckboxTree
- Outline — 大纲
- NestedList
- DirectoryListing — 目录列表视图
- Menu List
- Transfer List

---

## 14. 表格（Tables）

- Table（基础）
- DataTable — 排序、筛选、分页、选择
- VirtualTable — 百万行虚拟滚动
- DataGrid — 可编辑单元格（类 Excel）
- Spreadsheet — 电子表格（公式、合并单元格、冻结、填充柄）
- TreeTable — 树形表格
- PivotTable — 透视表
- ColumnResizer — 列宽拖拽
- ColumnReorder — 列拖拽排序
- ColumnVisibility — 列显示控制
- ColumnPinning — 冻结列
- RowSelection
- RowExpansion — 展开行详情
- RowGrouping — 分组行
- InlineRowEdit
- CellRenderer（文本 / 数字 / 标签 / 进度 / 迷你图 / 头像 / 链接）
- TableToolbar（搜索、过滤、导出、密度切换）
- FilterBuilder — 高级筛选构建器
- SortBuilder
- AggregationFooter — 汇总行
- TableDensity — 紧凑 / 标准 / 宽松
- ConditionalFormatting — 条件格式（红绿色阶）
- HeatmapTable
- ComparisonTable

---

## 15. 通用图表（Charts）

- LineChart
- AreaChart / StackedArea
- BarChart（水平 / 垂直 / 堆叠 / 分组）
- PieChart / DonutChart
- ScatterChart
- BubbleChart
- RadarChart
- HeatmapChart
- CalendarHeatmap（贡献图）
- Treemap
- Sunburst
- SankeyChart
- FunnelChart
- WaterfallChart
- BoxPlot
- Histogram
- ViolinPlot
- GanttChart
- NetworkGraph / ForceGraph
- ChordDiagram
- ParallelCoordinates
- Sparkline / SparkBar / SparkArea
- Bullet Chart
- ProgressChart
- RealtimeChart — 实时流数据图
- ChartLegend
- ChartTooltip
- ChartCrosshair
- ChartAxis / ChartGrid
- ChartZoom / Brush — 缩放与区间选择
- ChartAnnotation — 标注
- ChartExport

---

## 16. 金融与股票平台（Finance & Trading）

### 行情
- CandlestickChart — K 线图
- OHLCChart
- HeikinAshiChart
- LineQuoteChart — 分时图
- VolumeChart — 成交量
- DepthChart — 深度图
- MarketProfile / VolumeProfile
- RenkoChart / PointAndFigure
- MultiChartLayout — 多图联动布局
- ChartSync — 多图十字光标同步
- TimeRangeSelector（1m / 5m / 1H / 1D / 1W / 1M / YTD / 5Y）
- IntervalSelector
- ChartTypeSwitcher

### 技术分析
- IndicatorOverlay（MA / EMA / BOLL / VWAP）
- IndicatorPane（MACD / RSI / KDJ / OBV）
- IndicatorSelector
- DrawingTools — 画线工具（趋势线、水平线、斐波那契、矩形、文字标注）
- DrawingToolbar
- CompareSymbol — 叠加对比
- PriceAlertLine — 价格预警线

### 报价与盘口
- Ticker / TickerTape — 滚动行情条
- QuoteCard — 报价卡片
- PriceText — 涨跌颜色、闪烁
- PriceChangeBadge
- OrderBook — 买卖盘口（深度条）
- Level2 Quotes
- TimeAndSales / TradeTape — 逐笔成交
- MarketDepthLadder / DOM — 价格阶梯
- SpreadIndicator
- BidAskBar

### 交易
- OrderEntry / TradePanel — 下单面板（限价、市价、止损、条件单）
- QuickTradeButtons — 快速买卖
- PositionTable — 持仓
- OrderTable — 委托
- TradeHistoryTable — 成交记录
- PnLDisplay — 盈亏显示
- LeverageSlider
- MarginIndicator
- RiskMeter
- OrderConfirmDialog
- OptionChain — 期权链
- GreeksTable
- PayoffDiagram — 期权损益图

### 市场与资产
- Watchlist — 自选列表（分组、拖拽、实时刷新）
- SymbolSearch — 代码搜索
- SymbolBadge — 股票代码标签
- MarketHeatmap — 板块热力图
- SectorTreemap
- Screener / FilterPanel — 选股器
- MarketOverview — 大盘概览
- IndexCard
- MarketStatus — 开盘 / 收盘 / 盘前盘后
- TradingSessionClock
- EconomicCalendar
- EarningsCalendar
- NewsFeed — 财经新闻流
- SentimentGauge
- PortfolioSummary
- AssetAllocationChart
- PerformanceChart（收益曲线、回撤）
- DividendTable
- FinancialStatementTable — 财报表格
- CurrencyConverter
- CryptoWalletCard
- TransactionList
- Candle Countdown — K 线倒计时

---

## 17. 代码编辑器与 IDE（Code Editor & IDE）

### 编辑器核心
- CodeEditor — 语法高亮、多光标、选区、撤销重做
- EditorGutter — 行号、折叠箭头、断点、Git 标记、诊断图标
- LineNumbers（绝对 / 相对）
- FoldingControl
- Minimap
- Ruler / IndentGuide — 标尺、缩进参考线
- BracketMatcher / RainbowBrackets
- WhitespaceRenderer
- CursorRenderer（块状 / 线条 / 下划线、闪烁）
- SelectionHighlight
- MultiCursor
- VimModeIndicator
- StickyScroll — 作用域吸顶
- Breadcrumb（符号路径）
- InlayHints — 内联提示
- CodeLens — 代码上方操作
- InlineDecoration — 波浪线、背景色
- GhostText — AI 灰色补全文本
- InlineChat — 行内 AI 对话
- InlineDiff — 行内差异
- ReadOnlyBanner

### 智能提示
- CompletionMenu — 自动补全（图标、类型、文档预览）
- SignatureHelp — 参数提示
- HoverInfo — 悬停文档
- QuickFix / CodeActionMenu — 灯泡菜单
- RenameInput — 行内重命名
- PeekView — 查看定义 / 引用
- ReferencesPanel
- SymbolOutline
- GoToSymbol
- CallHierarchy / TypeHierarchy

### 搜索
- FindWidget — 查找（大小写、全词、正则）
- ReplaceWidget
- SearchPanel — 全局搜索结果（按文件分组）
- SearchResultItem（匹配高亮、预览）
- SearchFilters（包含 / 排除 glob）

### 面板
- ProblemsPanel / DiagnosticsList
- OutputPanel
- DebugConsole
- ExtensionsPanel / PluginList
- SettingsEditor（UI + JSON 双模式）
- KeybindingsEditor
- ThemePreview
- WelcomePage
- RecentProjects
- ProjectSwitcher
- TaskRunner / RunConfiguration

### 状态栏项
- CursorPosition（Ln / Col）
- LanguageMode
- Encoding
- LineEnding
- IndentSettings
- BranchIndicator
- LspStatus
- NotificationBell

---

## 18. 终端（Terminal）

- Terminal — 终端模拟器（ANSI 颜色、真彩、光标、滚动缓冲）
- TerminalTabs
- TerminalSplit
- TerminalToolbar
- TerminalSearch
- TerminalLink — 可点击路径 / URL
- ShellSelector
- CommandBlock — 命令分块（类 Warp）
- CommandHistory
- AnsiText — ANSI 文本渲染
- LogViewer — 日志查看器（级别过滤、跟随、高亮）
- ProcessList

---

## 19. 版本控制（Git / VCS）

- DiffViewer — 并排 / 统一视图
- InlineDiff
- ThreeWayMerge — 三方合并
- ConflictResolver
- ChangesList — 暂存 / 未暂存
- CommitInput — 提交信息输入
- CommitGraph — 提交图谱
- CommitList / CommitItem
- BranchSelector
- BranchList
- TagList
- StashList
- BlameView / GitBlameAnnotation
- FileHistory
- PullRequestCard
- ReviewComment
- DiffStat（+12 −3）
- GitStatusBadge（M / A / D / U）

---

## 20. 调试与运行（Debug）

- DebugToolbar（继续、单步、步入、步出、停止）
- BreakpointList
- BreakpointGutter
- CallStack
- VariablesPanel（树形展开）
- WatchPanel
- ThreadList
- MemoryViewer / HexViewer
- Disassembly
- PerformanceProfiler / Flamegraph
- TimelineProfiler
- NetworkInspector
- ConsoleREPL

---

## 21. 文档与富文本（Documents & Rich Text）

### 编辑
- RichTextEditor
- MarkdownEditor（所见即所得 / 源码 / 分屏）
- BlockEditor — 块编辑器（类 Notion）
- Block 类型：段落、标题、列表、待办、引用、代码、分割线、表格、图片、视频、嵌入、公式、Callout、折叠块、列布局、同步块
- BlockHandle — 块拖拽手柄
- SlashMenu — 斜杠命令
- FloatingFormatToolbar — 选中格式工具栏
- FixedFormatToolbar — 固定工具栏
- LinkEditor — 链接编辑浮窗
- MentionMenu — @ 提及
- EmojiAutocomplete
- TableEditor — 表格编辑（增删行列、合并）
- ImageEditorBlock（缩放、对齐、说明文字）
- MathEditor / LatexBlock
- MermaidBlock / DiagramBlock
- CodeBlockEditor
- TodoItem / Checklist
- PlaceholderText — 空块提示
- WordCount / CharacterCount
- ReadingTime

### 阅读
- MarkdownRenderer
- DocumentViewer
- PDFViewer（缩略图、搜索、缩放、标注）
- EpubReader
- OfficePreview（docx / xlsx / pptx 预览）
- PageThumbnailList
- TableOfContents（滚动联动高亮）
- DocumentOutline
- Footnote
- Glossary / TooltipTerm
- ReadingProgress
- FocusMode / ZenMode
- PrintPreview

### 知识库
- PageTree — 页面层级树
- Backlinks — 反向链接
- GraphView — 双链关系图
- PageCover / PageIcon
- PageProperties — 页面属性（数据库字段）
- DatabaseView（表格 / 看板 / 画廊 / 日历 / 时间线视图）
- TemplatePicker
- TrashBin / 回收站
- VersionHistory — 版本历史
- PageHistoryDiff
- Favorites / Pinned

---

## 22. 协作（Collaboration）

- PresenceAvatars — 在线协作者
- RemoteCursor — 他人光标
- RemoteSelection — 他人选区
- CommentThread
- CommentBubble / CommentMarker
- CommentSidebar
- Reply / ReactionPicker
- Reactions（表情回应）
- Annotation / Highlight 批注
- SuggestionMode — 修订模式
- TrackChanges — 修订记录
- ShareDialog — 分享 & 权限
- PermissionSelect（可查看 / 可评论 / 可编辑）
- InviteInput
- AccessList
- ActivityLog
- FollowMode — 跟随他人视图
- LiveIndicator

---

## 23. AI 对话平台（AI Chat）

### 消息
- ChatContainer
- MessageList（虚拟化、自动滚动到底部、新消息提示）
- MessageBubble（用户 / 助手 / 系统 / 错误）
- MessageAvatar
- MessageHeader（模型名、时间）
- MessageFooter
- StreamingText — 流式渲染（平滑、光标）
- StreamingMarkdown — 增量 Markdown 解析渲染
- StreamingCursor — 流式输出光标
- CodeBlock（语言标签、复制、下载、运行、折叠、应用到文件）
- MathBlock
- TableBlock
- MermaidRenderer
- ImageMessage / ImageGrid
- FileMessage
- AudioMessage
- VideoMessage
- LinkPreviewCard
- QuoteReply — 引用回复
- MessageActions（复制、重新生成、编辑、点赞、点踩、朗读、分享）
- MessageEditor — 编辑已发送消息
- BranchNavigator（< 2/3 >）— 分支切换
- RegenerateButton
- StopGeneratingButton
- ContinueButton
- ErrorMessage / RetryMessage
- RateLimitNotice
- FeedbackForm — 反馈表单
- ScrollToBottomButton
- DateSeparator
- ThinkingBlock / ReasoningPanel — 可折叠思考过程
- ThinkingIndicator — 思考中动画
- ThinkingDuration（已思考 12 秒）

### 引用与检索
- CitationBadge / InlineCitation（[1]）
- SourceCard
- SourceList / SourcesPanel
- SearchProgress — 搜索中（显示正在搜索的关键词）
- WebResultCard
- DocumentChunkPreview

### 输入
- PromptInput / Composer（多行、快捷键发送、Shift 换行）
- AttachmentButton
- AttachmentChip / AttachmentPreview
- PasteImagePreview
- DragDropOverlay — 拖入文件遮罩
- VoiceInputButton
- VoiceWaveform / RecordingIndicator
- SlashCommandMenu
- PromptTemplateMenu
- ContextMentionMenu（@文件 / @文档 / @网页）
- ContextChips — 已附加上下文
- ModelSelector（能力标签、上下文长度）
- ModeSelector（对话 / 搜索 / 深度研究 / Agent）
- ToolToggleMenu（启用联网、代码执行等）
- TemperatureSlider / ParameterPanel
- SystemPromptEditor
- TokenCounter
- ContextWindowMeter — 上下文使用率
- CostEstimator
- SendButton（发送 / 停止 两态）
- InputHint（按 Enter 发送）

### 会话管理
- ConversationList（按时间分组：今天、昨天、7 天内）
- ConversationItem（重命名、删除、置顶、导出）
- ConversationSearch
- NewChatButton
- FolderList / ProjectList
- ProjectKnowledgePanel — 项目知识库
- SharedConversationView
- ConversationExport

### 引导
- WelcomeScreen / Greeting
- SuggestionChips / StarterPrompts
- PromptLibrary
- FollowUpSuggestions — 追问建议
- CapabilityCards

---

## 24. AI Agent 与工具调用（Agent）

- ToolCallCard — 工具调用（名称、参数、结果、耗时、状态）
- ToolCallGroup — 多个调用折叠
- ToolApprovalDialog — 执行前审批
- PermissionPrompt（允许一次 / 始终允许 / 拒绝）
- AgentStepList / TaskTimeline — 执行步骤
- AgentPlan / TodoList — 计划清单（实时勾选）
- AgentStatus（规划中 / 执行中 / 等待 / 完成 / 失败）
- AgentProgress
- SubAgentTree — 子 Agent 层级
- FileChangeCard — 文件变更预览（接受 / 拒绝）
- MultiFileDiff Review
- CommandExecutionCard — 命令执行 + 输出
- BrowserPreview / ScreenshotStream — Agent 浏览器画面
- ComputerUseViewer
- ArtifactPanel — 产物面板（代码、文档、网页预览）
- ArtifactVersionSwitcher
- LivePreview — 实时预览
- SandboxStatus
- MCPServerList / ConnectorList
- ToolRegistry Panel
- MemoryPanel — Agent 记忆管理
- CheckpointList / Rewind — 检查点回滚
- InterruptButton
- HumanInputRequest — Agent 请求用户输入
- TokenUsageChart
- CostBreakdown
- TraceViewer — 调用链追踪
- EvalResultTable

---

## 25. AI 创作与生成（Generative）

- ImageGenerationPanel
- GenerationQueue — 生成队列
- GenerationGrid — 结果网格
- VariationPicker
- PromptEnhancer
- NegativePromptInput
- StylePresetPicker
- SeedInput
- AspectRatioPicker
- InpaintCanvas — 局部重绘画布
- MaskBrush
- ImageCompare
- AudioGenerationPlayer
- VideoGenerationTimeline
- TTSVoicePicker
- ModelCard — 模型信息卡
- ModelDownloadManager — 本地模型下载
- ModelStatus（已加载 / 加载中 / GPU 占用）
- HardwareMonitor（GPU / VRAM / CPU）
- FineTuneJobCard
- DatasetViewer
- EmbeddingVisualizer（2D / 3D 散点）
- PromptPlayground — 提示词调试台
- PromptDiff / PromptVersionHistory
- ABCompareView — 多模型对比

---

## 26. 媒体（Media）

- ImageViewer（缩放、平移、旋转、适配）
- ImageCropper
- ImageAnnotator
- ImageThumbnail
- Lightbox / Gallery Viewer
- VideoPlayer（播放、进度、倍速、字幕、全屏、画中画）
- VideoTimeline / Scrubber
- VideoThumbnailStrip
- AudioPlayer
- AudioWaveform
- AudioSpectrum / Visualizer
- VolumeControl
- PlaybackSpeedControl
- MediaControls
- Playlist
- SubtitleEditor
- ScreenRecorderControls
- CameraPreview
- MicLevelMeter
- DeviceSelector（麦克风 / 摄像头 / 扬声器）
- 3DModelViewer

---

## 27. 文件管理（Files）

- FileExplorer — 文件浏览器（列表 / 图标 / 分栏视图）
- FileGrid
- FileList
- FileItem（图标、名称、大小、修改时间）
- FileIcon — 按扩展名自动图标
- FilePreview / QuickLook
- FileBreadcrumb / PathBar
- FolderTree
- FileContextMenu
- RenameInline
- DragDropFiles
- FileOperationProgress — 复制 / 移动进度
- TransferQueue — 上传下载队列
- DownloadManager
- StorageUsage
- RecentFiles
- Favorites / Bookmarks
- FileSearch
- DuplicateFinder 视图
- ArchiveViewer（zip 内容）
- HexViewer

---

## 28. 即时通讯（IM / Messaging）

- ChannelList / ChatList
- ChannelItem（未读数、静音、置顶）
- DirectMessageItem
- MessageThread
- ThreadPanel — 话题侧栏
- ChatMessage（连续消息合并、时间戳）
- UnreadDivider — 新消息分割线
- MessageReactions
- ReadReceipt — 已读回执
- TypingIndicator
- MemberList
- UserProfileCard
- OnlineStatus（在线 / 离开 / 忙碌 / 离线）
- StatusSetter — 自定义状态
- VoiceCallBar
- VideoCallGrid
- ScreenShareView
- CallControls（静音、摄像头、挂断、共享）
- ParticipantTile
- IncomingCallDialog
- StickerPicker
- GifPicker
- PinnedMessages
- ChannelHeader
- HuddleIndicator

---

## 29. 邮件（Mail）

- MailboxList / FolderList
- MailList（未读粗体、星标、附件图标）
- MailItem
- MailReader
- MailComposer（收件人、抄送、密送、附件）
- RecipientInput
- MailThreadView — 会话视图
- QuotedText — 折叠引用
- SignatureEditor
- LabelPicker
- SnoozePicker
- ScheduleSend
- InboxZeroState

---

## 30. 日历与日程（Calendar）

- CalendarMonthView
- CalendarWeekView
- CalendarDayView
- AgendaView
- YearView
- MiniCalendar
- EventCard / EventChip
- EventEditor
- EventPopover
- TimeGrid — 时间网格（拖拽创建、调整时长）
- AllDayRow
- CurrentTimeIndicator
- RecurrenceEditor — 重复规则
- AvailabilityPicker
- TimezoneOverlay
- ReminderPicker
- AttendeeList

---

## 31. 项目与任务管理（Project Management）

- KanbanBoard / KanbanColumn / KanbanCard
- TaskList
- TaskItem（复选、优先级、截止日期、负责人）
- TaskDetailPanel
- SubtaskList
- GanttChart / TimelineView
- Roadmap
- Sprint Board
- Burndown Chart
- PriorityIndicator
- StatusSelect
- AssigneePicker
- DueDatePicker
- LabelManager
- IssueCard
- IssueIdBadge（ENG-123）
- MilestoneProgress
- WorkloadView
- TimeTracker
- Pomodoro Timer

---

## 32. 设计与画布工具（Canvas & Design）

- InfiniteCanvas — 无限画布（缩放、平移）
- CanvasGrid / Guides — 网格与参考线
- Ruler — 标尺
- SelectionBox — 选框
- TransformHandles — 缩放旋转手柄
- SnapIndicator — 吸附提示
- LayerPanel — 图层面板
- AssetPanel
- InspectorPanel — 属性检查器
- AlignmentToolbar
- ToolPalette — 工具箱
- ZoomControls / ZoomIndicator
- MiniMap（画布）
- ArtboardFrame
- ShapeTools（矩形、椭圆、多边形、线条、箭头）
- PenTool / PathEditor
- TextTool
- BrushTool / BrushSettings
- ColorPanel
- GradientPanel
- ShadowEditor
- BorderEditor
- TypographyPanel
- ConstraintsEditor
- AutoLayoutControls
- ExportPanel
- NodeEditor / NodeGraph — 节点编辑器（ComfyUI / 蓝图风格）
- Node / Port / Edge / Connection
- FlowChart / DiagramEditor
- MindMap
- Whiteboard
- StickyNote
- Connector Line
- HistoryPanel — 操作历史

---

## 33. 数据库与开发工具（DB & Dev Tools）

- ConnectionManager — 数据库连接管理
- ConnectionForm
- SchemaTree — 库 / 表 / 字段树
- QueryEditor — SQL 编辑器
- QueryResultGrid
- QueryHistory
- QueryPlanViewer — 执行计划
- TableStructureEditor
- ERDiagram — 实体关系图
- IndexManager
- RedisKeyBrowser
- DocumentViewer（MongoDB / JSON）
- JsonViewer / JsonTree（折叠、路径复制、搜索）
- JsonEditor
- YamlViewer
- XmlViewer
- ApiRequestBuilder（方法、URL、Headers、Body、Auth）
- ResponseViewer（状态码、耗时、大小、美化）
- HeadersTable
- QueryParamsEditor
- EnvironmentSelector — 环境变量切换
- CollectionTree — 请求集合
- WebSocketConsole
- GraphQLExplorer
- RegexTester
- Base64 / Encoder 工具面板
- ColorContrastChecker
- ContainerList（Docker）
- PodList（K8s）
- ResourceMonitor

---

## 34. 监控与仪表盘（Dashboard & Monitoring）

- Dashboard Grid — 可拖拽调整的仪表盘网格
- Widget / DashboardCard
- StatCard
- MetricsChart
- AlertList
- IncidentCard
- UptimeBar — 可用性条
- ServiceStatus
- HealthCheck Indicator
- LogStream
- TraceWaterfall
- EventStream
- ResourceGauge（CPU / 内存 / 磁盘 / 网络）
- SystemMonitor
- ProcessTable
- NetworkGraph（实时速率）
- MapView / GeoHeatmap
- WorldMap
- RefreshIntervalSelector
- DashboardFilterBar

---

## 35. 设置与偏好（Settings）

- SettingsLayout — 左侧分类 + 右侧内容
- SettingsSection
- SettingsRow（标题、描述、控件）
- SettingsSearch
- ThemeSelector（亮 / 暗 / 跟随系统）
- AccentColorPicker
- FontSizeControl
- FontFamilySelect
- DensitySelector
- LanguageSelector
- KeyboardShortcutsList
- ShortcutEditor
- ProxySettings
- StorageSettings / CacheClear
- PrivacySettings
- NotificationSettings
- StartupSettings
- ImportExportSettings
- ResetToDefault
- AdvancedSettings / FeatureFlags
- DeveloperMode Toggle

---

## 36. 账户与用户（Account）

- LoginForm
- SignupForm
- OAuthButtons
- MagicLinkForm
- TwoFactorInput
- ForgotPassword
- ProfileCard
- ProfileEditor
- AccountSwitcher — 多账户切换
- WorkspaceSwitcher / OrgSwitcher
- UserMenu — 头像下拉菜单
- SessionList — 登录设备
- ApiKeyManager
- SubscriptionCard / PlanBadge
- UsageQuota
- BillingHistory
- PaymentMethodForm
- UpgradePrompt / Paywall
- TeamMemberTable
- RoleSelector
- InvitationList

---

## 37. 引导与帮助（Onboarding & Help）

- OnboardingWizard
- WelcomeTour
- Coachmark / Hotspot
- FeatureHighlight / NewBadge
- Checklist（新手任务）
- HelpTooltip（? 图标）
- HelpPanel / DocsSidebar
- KeyboardShortcutCheatsheet
- WhatsNewDialog
- FeedbackWidget
- ContactSupport
- InlineHelp
- EmptyStateWithAction

---

## 38. 交互原语（Interaction）

- Draggable
- Droppable / DropTarget
- DragOverlay / DragPreview
- DragGhost
- Sortable / SortableContext
- Resizable
- Rotatable
- Selectable / SelectionArea — 框选
- RubberBandSelection
- LongPress
- DoubleClick
- Gesture（滚轮缩放、触控板捏合、双指滑动）
- PanZoom
- Hotkeys / KeyBinding — 快捷键绑定与作用域
- KeyChord — 组合键序列（Ctrl+K Ctrl+S）
- Undo / Redo Manager
- InfiniteScroll Trigger
- VirtualScroller（底层）
- ScrollSync — 多区域滚动同步
- Autofocus
- RovingTabIndex — 方向键焦点管理
- TypeAhead — 按键快速定位
- FocusVisible

---

## 39. 主题与样式系统（Theme）

- ThemeProvider
- ThemeSwitcher
- ColorTokens / DesignTokens
- ThemeEditor — 主题编辑器
- ThemeImporter（VS Code 主题兼容）
- SyntaxThemes — 代码高亮主题
- IconThemeProvider
- DensityProvider
- RadiusProvider
- FontProvider
- HighContrastMode
- ReducedMotionProvider
- Platform Style（macOS / Windows / Linux 自适应）
- VibrancyBackground / Mica / Acrylic — 系统毛玻璃
- Elevation / Shadow 系统

---

## 40. 国际化与无障碍（i18n & a11y）

- I18nProvider
- Trans / LocalizedText
- LocaleNumber / LocaleDate
- RTL 布局支持
- LiveRegion / Announcer — 屏幕阅读器播报
- SkipLink
- AccessibleIcon
- AriaLabel 辅助
- KeyboardNavigationHelper
- ScreenReaderOnly
- ColorBlindSafe Palette

---

## 41. 地图与地理（Maps）

- MapView（瓦片）
- MapMarker
- MapPopup
- MapCluster
- RouteLine
- GeoJSON Layer
- ChoroplethMap
- LocationPicker
- CoordinateDisplay

---

## 42. 杂项与工具组件（Misc）

- Clock / WorldClock
- Stopwatch
- Calculator
- UnitConverter
- QRCodeScanner
- BarcodeDisplay
- Captcha
- Signature
- Rating Widget
- Poll / Survey
- Quiz / Flashcard
- ChangeLog Viewer
- LicenseViewer
- Terms / Consent Dialog
- CookieBanner（嵌入 Web 内容时）
- WebView 容器
- IframeEmbed / EmbedCard
- HTMLPreview
- PrintButton
- ExportDialog（PDF / PNG / CSV / JSON）
- ImportDialog（字段映射）
- CSVImporter
- ShareSheet

---

## 43. 开发与质量（Library Tooling）

- ComponentGallery / Storybook — 组件展示应用
- PlaygroundPanel — 属性实时调试
- InspectorOverlay — 布局检查（边框、尺寸）
- FPSMeter / PerfOverlay
- RenderCounter
- EventLogger
- DesignTokenViewer
- IconBrowser
- A11yChecker
- ScreenshotTest Harness

---

## 44. 渲染表面（Rendering）

- RenderSurface — 宿主按盒子的设备像素绘出帧
- FramebufferView / OffscreenRenderSurface / CustomRendererSurface / TextureView → RenderSurface
- ShaderView / GpuSurface / WebGPUSurface / Compute Visualization / GPU Particle View — 宿主自带 GPU，读回帧交给 RenderSurface
- CameraTexture / GStreamerTexture / ExternalTexture — 宿主的帧交给 RenderSurface
- ZeroCopyTexture / SharedTextureView / MetalSurface / OpenGLSurface / VulkanSurface / D3D11/D3D12Surface — gpui 不共享设备
- 3D Scene / 3D Viewport → media::ModelViewer
