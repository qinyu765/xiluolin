-- 测试脚本：验证 personas 表结构
.schema personas

-- 查询所有人格
SELECT id, name, icon, is_default FROM personas;

-- 统计人格数量
SELECT COUNT(*) as total FROM personas;
