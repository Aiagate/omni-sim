"""
シミュレーション定数定義
"""

# === 艦船パラメータ ===

# 艦種別基礎パラメータ
SHIP_CLASSES = {
    "destroyer": {  # 駆逐艦
        "hp": 100,
        "attack": 20,
        "defense": 10,
        "speed": 5.0,
        "max_fuel": 5000.0,  # 最大燃料（kg）
        "max_ammo": 200,  # 最大弾薬（発）
        "fuel_consumption_rate": 10.0,  # 燃料消費率（kg/tick）
        "ammo_per_shot": 1,  # 1回の攻撃での弾薬消費（発）
    },
    "cruiser": {  # 巡洋艦
        "hp": 200,
        "attack": 40,
        "defense": 20,
        "speed": 3.0,
        "max_fuel": 20000.0,  # 最大燃料（kg）
        "max_ammo": 400,  # 最大弾薬（発）
        "fuel_consumption_rate": 20.0,  # 燃料消費率（kg/tick）
        "ammo_per_shot": 2,  # 1回の攻撃での弾薬消費（発）
    },
    "battleship": {  # 戦艦
        "hp": 500,
        "attack": 100,
        "defense": 50,
        "speed": 2.0,
        "max_fuel": 100000.0,  # 最大燃料（kg）
        "max_ammo": 1000,  # 最大弾薬（発）
        "fuel_consumption_rate": 50.0,  # 燃料消費率（kg/tick）
        "ammo_per_shot": 5,  # 1回の攻撃での弾薬消費（発）
    },
}

# === 戦闘パラメータ ===

# 基礎命中率（0.0～1.0）
BASE_HIT_RATE = 0.7

# ダメージ計算式: max(attack - defense, min_damage)
MIN_DAMAGE = 5  # 最低ダメージ

# === 索敵パラメータ ===

# 基礎索敵範囲
BASE_DETECTION_RANGE = 100.0

# === シミュレーション設定 ===

# 最大tick数（無限ループ防止）
MAX_TICKS = 1000

# ログ出力レベル
LOG_LEVEL = "INFO"  # DEBUG, INFO, WARNING, ERROR

# === リソース単位表記 ===

# 燃料単位
FUEL_UNIT = "kg"  # キログラム

# 弾薬単位
AMMO_UNIT = "rounds"  # 発（ラウンド）
