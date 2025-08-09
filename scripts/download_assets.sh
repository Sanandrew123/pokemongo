#!/bin/bash
# 宝可梦素材下载脚本
# 开发心理：自动化素材获取，从开源项目和API获取合法资源
# 优先使用Pokemon Showdown的开源素材和PokeAPI数据

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🎮 Pokemon GO 素材下载器${NC}"
echo "======================================"

# 检查依赖
check_dependencies() {
    echo -e "${YELLOW}检查依赖工具...${NC}"
    
    if ! command -v curl &> /dev/null; then
        echo -e "${RED}错误: 需要安装 curl${NC}"
        exit 1
    fi
    
    if ! command -v jq &> /dev/null; then
        echo -e "${YELLOW}警告: 建议安装 jq 用于JSON处理${NC}"
        # 尝试安装jq
        if command -v apt-get &> /dev/null; then
            sudo apt-get update && sudo apt-get install -y jq
        fi
    fi
    
    echo -e "${GREEN}✓ 依赖检查完成${NC}"
}

# 创建目录结构
create_directories() {
    echo -e "${YELLOW}创建资源目录...${NC}"
    
    mkdir -p assets/textures/pokemon/{sprites,portraits,animations}
    mkdir -p assets/audio/{music,sfx/pokemon_cries}
    mkdir -p assets/data/{base,creatures/templates}
    
    echo -e "${GREEN}✓ 目录创建完成${NC}"
}

# 下载Pokemon Showdown精灵图
download_pokemon_sprites() {
    echo -e "${YELLOW}下载宝可梦精灵图...${NC}"
    
    local base_url="https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon"
    local sprite_dir="assets/textures/pokemon/sprites"
    
    # 下载前151只宝可梦的精灵图
    for i in {1..151}; do
        echo -e "${YELLOW}下载 Pokemon #$i...${NC}"
        
        # 正面精灵图
        curl -s -L "${base_url}/${i}.png" -o "${sprite_dir}/front_${i}.png" || {
            echo -e "${RED}警告: Pokemon #$i 正面图下载失败${NC}"
        }
        
        # 背面精灵图
        curl -s -L "${base_url}/back/${i}.png" -o "${sprite_dir}/back_${i}.png" || {
            echo -e "${RED}警告: Pokemon #$i 背面图下载失败${NC}"
        }
        
        # 闪光形态
        curl -s -L "${base_url}/shiny/${i}.png" -o "${sprite_dir}/shiny_${i}.png" || {
            echo -e "${RED}警告: Pokemon #$i 闪光图下载失败${NC}"
        }
        
        # 避免请求过于频繁
        sleep 0.1
    done
    
    echo -e "${GREEN}✓ 精灵图下载完成${NC}"
}

# 下载Pokemon数据
download_pokemon_data() {
    echo -e "${YELLOW}下载宝可梦数据...${NC}"
    
    local data_dir="assets/data/base"
    local api_base="https://pokeapi.co/api/v2"
    
    # 下载基础Pokemon数据
    echo "下载Pokemon物种数据..."
    curl -s "${api_base}/pokemon-species?limit=151" > "${data_dir}/species.json"
    
    # 下载技能数据
    echo "下载技能数据..."
    curl -s "${api_base}/move?limit=165" > "${data_dir}/moves.json"
    
    # 下载属性数据
    echo "下载属性数据..."
    curl -s "${api_base}/type" > "${data_dir}/types.json"
    
    # 下载道具数据
    echo "下载道具数据..."
    curl -s "${api_base}/item?limit=100" > "${data_dir}/items.json"
    
    echo -e "${GREEN}✓ 基础数据下载完成${NC}"
}

# 下载音效文件 (示例)
download_audio_samples() {
    echo -e "${YELLOW}下载音频示例...${NC}"
    
    local audio_dir="assets/audio"
    
    # 下载一些基础音效 (使用开源音效)
    # 这里使用freesound.org的开源音效作为占位符
    
    # 创建基础音效文件 (空文件作为占位符)
    touch "${audio_dir}/sfx/click.ogg"
    touch "${audio_dir}/sfx/select.ogg" 
    touch "${audio_dir}/sfx/back.ogg"
    touch "${audio_dir}/music/title_theme.ogg"
    touch "${audio_dir}/music/battle_theme.ogg"
    
    # 创建Pokemon叫声占位符
    for i in {1..151}; do
        touch "${audio_dir}/sfx/pokemon_cries/cry_${i}.ogg"
    done
    
    echo -e "${GREEN}✓ 音频文件占位符创建完成${NC}"
    echo -e "${YELLOW}注意: 实际音频文件需要手动获取合法来源${NC}"
}

# 生成基础数据文件
generate_base_data() {
    echo -e "${YELLOW}生成基础配置文件...${NC}"
    
    # 生成Pokemon模板
    cat > assets/data/creatures/templates/fire_type.json << 'EOF'
{
  "template_id": "fire_base",
  "version": "1.0",
  "base_stats": {
    "hp_range": [45, 78],
    "attack_range": [49, 84], 
    "defense_range": [49, 78],
    "speed_range": [45, 65]
  },
  "type_primary": "Fire",
  "abilities": ["Blaze"],
  "growth_rate": "medium_slow"
}
EOF

    # 生成水系模板
    cat > assets/data/creatures/templates/water_type.json << 'EOF'
{
  "template_id": "water_base", 
  "version": "1.0",
  "base_stats": {
    "hp_range": [50, 80],
    "attack_range": [48, 83],
    "defense_range": [65, 100],
    "speed_range": [43, 78]
  },
  "type_primary": "Water",
  "abilities": ["Torrent"],
  "growth_rate": "medium_slow"
}
EOF

    # 生成草系模板
    cat > assets/data/creatures/templates/grass_type.json << 'EOF'
{
  "template_id": "grass_base",
  "version": "1.0", 
  "base_stats": {
    "hp_range": [45, 80],
    "attack_range": [49, 82],
    "defense_range": [49, 83],
    "speed_range": [45, 80]
  },
  "type_primary": "Grass",
  "abilities": ["Overgrow"],
  "growth_rate": "medium_slow"
}
EOF

    echo -e "${GREEN}✓ 基础配置文件生成完成${NC}"
}

# 验证下载结果
verify_assets() {
    echo -e "${YELLOW}验证资源完整性...${NC}"
    
    local sprite_count=$(find assets/textures/pokemon/sprites -name "*.png" | wc -l)
    local data_count=$(find assets/data -name "*.json" | wc -l)
    
    echo "精灵图数量: $sprite_count"
    echo "数据文件数量: $data_count"
    
    if [ "$sprite_count" -gt 100 ]; then
        echo -e "${GREEN}✓ 精灵图下载完成度良好${NC}"
    else
        echo -e "${YELLOW}警告: 精灵图数量较少，可能存在下载失败${NC}"
    fi
    
    if [ "$data_count" -gt 5 ]; then
        echo -e "${GREEN}✓ 数据文件完整${NC}"
    else
        echo -e "${RED}错误: 数据文件不完整${NC}"
    fi
}

# 主函数
main() {
    echo -e "${GREEN}开始下载Pokemon游戏素材...${NC}"
    echo "这可能需要几分钟时间，请耐心等待..."
    echo ""
    
    check_dependencies
    create_directories
    download_pokemon_sprites
    download_pokemon_data
    download_audio_samples
    generate_base_data
    verify_assets
    
    echo ""
    echo -e "${GREEN}🎉 素材下载完成！${NC}"
    echo -e "${YELLOW}提示: 部分音频文件为占位符，需要手动获取合法音源${NC}"
    echo -e "${YELLOW}建议: 检查assets目录确认所有文件正确下载${NC}"
}

# 执行主函数
main "$@"