const Sequencer = require('@jest/test-sequencer').default;

class SolanaTestSequencer extends Sequencer {
  /**
   * 自定义测试执行顺序，确保系统初始化测试优先执行
   * 避免Solana账户状态冲突
   */
  sort(tests) {
    // 定义测试执行优先级
    const testPriority = {
      'system-initialization': 1,  // 系统初始化最优先
      'id-generator': 2,           // ID生成器次之
      'merchant': 3,               // 商户管理
      'product': 4,                // 产品管理
      'search': 5,                 // 搜索功能
      'index-management': 6,       // 索引管理
      'performance': 7,            // 性能测试
      'unit': 8,                   // 单元测试最后
    };

    return tests.sort((testA, testB) => {
      // 获取测试文件名
      const nameA = this.getTestName(testA.path);
      const nameB = this.getTestName(testB.path);

      // 获取优先级
      const priorityA = this.getTestPriority(nameA, testPriority);
      const priorityB = this.getTestPriority(nameB, testPriority);

      // 按优先级排序
      if (priorityA !== priorityB) {
        return priorityA - priorityB;
      }

      // 相同优先级按文件名排序
      return nameA.localeCompare(nameB);
    });
  }

  /**
   * 从文件路径提取测试名称
   */
  getTestName(filePath) {
    const fileName = filePath.split('/').pop() || '';
    return fileName.replace(/\.(spec|test)\.(ts|js)$/, '');
  }

  /**
   * 获取测试优先级
   */
  getTestPriority(testName, priorityMap) {
    // 查找匹配的优先级
    for (const [key, priority] of Object.entries(priorityMap)) {
      if (testName.includes(key)) {
        return priority;
      }
    }
    // 默认优先级
    return 999;
  }

  /**
   * 支持测试分片（如果需要）
   */
  shard(tests, { shardIndex, shardCount }) {
    const shardSize = Math.ceil(tests.length / shardCount);
    const start = shardIndex * shardSize;
    const end = start + shardSize;
    
    return tests.slice(start, end);
  }
}

module.exports = SolanaTestSequencer;
