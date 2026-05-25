import i18n from "i18next";
import { initReactI18next } from "react-i18next";

export const resources = {
	zh: {
		translation: {
			app: {
				name: "ACMind",
			},
			nav: {
				dashboard: "仪表盘",
				problems: "题目",
				reports: "报告",
				knowledge: "知识",
				settings: "设置",
				expandSidebar: "展开侧边栏",
				collapseSidebar: "收起侧边栏",
			},
			dashboard: {
				title: "仪表盘",
				subtitle: "你的 ACM 训练概览",
				totalProblems: "题目总数",
				acRate: "AC 率",
				gettingStarted: "开始使用",
				empty: "你的 ACM 训练库还是空的。先添加第一道题吧！",
				step1: "进入题目页，点击添加题目",
				step2: "记录提交记录（WA 和 AC 代码）",
				step3: "添加题解笔记和 AI 分析",
				step4: "在这里查看训练报告",
			},
			problems: {
				title: "题目",
				recorded: "{{count}} 道题已记录",
				add: "添加题目",
				addNew: "添加新题目",
				problemTitle: "标题 *",
				source: "来源",
				problemId: "题目 ID",
				difficulty: "难度",
				difficultyRating: "难度（rating）",
				tags: "标签",
				tagsComma: "标签（逗号分隔）",
				statement: "题面（Markdown，可选）",
				actions: "操作",
				placeholderTitle: "题目标题",
				placeholderStatement: "在这里粘贴题面...",
				search: "按标题、来源或标签搜索...",
				adding: "添加中...",
				deleteConfirm: "确定删除这道题吗？",
				empty: "没有找到题目。",
			},
			common: {
				cancel: "取消",
				save: "保存",
				generate: "生成",
				close: "关闭",
			},
		},
	},
} as const;

i18n.use(initReactI18next).init({
	resources,
	lng: "zh",
	fallbackLng: "zh",
	interpolation: {
		escapeValue: false,
	},
});

export default i18n;
