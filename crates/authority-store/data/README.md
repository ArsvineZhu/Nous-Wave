# LexicalRef 词表 v1

`lexical-words-v1.txt` 是冻结的 4096 词编码表，顺序和内容属于持久化寻址合同。

词语来源：[EFF Long Wordlist](https://www.eff.org/files/2016/07/18/eff_large_wordlist.txt)，作者 Electronic Frontier Foundation / Joseph Bonneau。依据 [EFF copyright policy](https://www.eff.org/copyright)，采用 [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/)。本项目作了子集选择、保留词/敏感词过滤、排序和 tokenizer 评估；EFF 未为本实现背书。

选择过程优先较短的 lowercase ASCII 单词，并减少单字符编辑的近似拼写。最终 4096 词仅用于随机不透明地址；词义不参与对象编码。唯一性由 Authority UNIQUE constraint 和碰撞重试保证，地址不用于认证。

`lexical-words-v1-measurement.json` 记录实际 source/table 摘要及 `cl100k_base`、`o200k_base` 两种编码器的样本结果。其他 tokenizer、发音混淆和真人复制错误率未宣称实测。
