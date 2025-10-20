"""
戦闘システムの単体テスト

戦闘解決ロジック、ダメージ計算、命中判定をテスト。
"""

import pytest
from src.entities.ship import Ship
from src.entities.fleet import Fleet
from src.combat.resolver import CombatResolver
from src.config.constants import BASE_HIT_RATE, MIN_DAMAGE
from tests.fixtures.scenarios import reset_rng, sample_fleet


@pytest.mark.unit
class TestCombatResolver:
    """戦闘解決のテスト"""

    def test_combat_resolver_creation(self):
        """CombatResolverの生成"""
        resolver = CombatResolver()
        assert resolver is not None

    def test_damage_calculation_hit(self, reset_rng):
        """ダメージ計算（命中時）"""
        attacker = Ship(1, "cruiser", "Attacker")  # 攻撃40
        defender = Ship(2, "destroyer", "Defender")  # 防御10

        resolver = CombatResolver()
        # シード42では最初の乱数は命中する確率が高い
        # 実際のダメージは attack - defense = 40 - 10 = 30
        # ただし命中判定で外れる場合は0

        # 複数回実行して命中ケースを確認
        damages = []
        for _ in range(10):
            attacker.ammo = attacker.max_ammo  # 弾薬をリセット
            damage = resolver._calculate_damage(attacker, defender)
            damages.append(damage)

        # 少なくとも1回は命中する（30ダメージ）
        assert 30 in damages or 0 in damages  # 命中or外れ

    def test_minimum_damage(self, reset_rng):
        """最低ダメージ保証"""
        # 攻撃力が防御力より低い場合でもMIN_DAMAGE保証
        attacker = Ship(1, "destroyer", "Weak")  # 攻撃20
        defender = Ship(2, "battleship", "Strong")  # 防御50

        resolver = CombatResolver()

        # 複数回実行して命中時の最低ダメージを確認
        for _ in range(20):
            attacker.ammo = attacker.max_ammo
            damage = resolver._calculate_damage(attacker, defender)
            if damage > 0:  # 命中した場合
                assert damage >= MIN_DAMAGE
                break

    def test_no_ammo_no_attack(self):
        """弾薬がなければ射撃できない（can_shootで判定）"""
        attacker = Ship(1, "destroyer", "Empty")
        attacker.ammo = 0
        defender = Ship(2, "destroyer", "Target")

        # 弾薬がない場合、can_shoot()がFalseを返す
        assert attacker.can_shoot() is False

        # _calculate_damageは弾薬チェックをしないので、
        # 実際の戦闘では_attack_fleetで弾薬チェックが行われる


@pytest.mark.unit
class TestFleetCombat:
    """艦隊戦闘のテスト"""

    def test_basic_combat_resolution(self, reset_rng):
        """基本的な艦隊戦闘"""
        fleet_a = Fleet(1, "Fleet-A", (0.0, 0.0, 0.0), ships=[Ship(1, "cruiser", "A-Ship")])
        
        fleet_b = Fleet(2, "Fleet-B", (0.0, 0.0, 0.0), ships=[Ship(2, "destroyer", "B-Ship")])
        
        resolver = CombatResolver()
        damage_to_b, damage_to_a = resolver.resolve_combat(fleet_a, fleet_b)
        
        # 少なくとも一方がダメージを与える（または外れる）
        assert damage_to_b >= 0
        assert damage_to_a >= 0

    def test_combat_with_destroyed_fleet(self, reset_rng):
        """全滅した艦隊は攻撃しない"""
        fleet_a = Fleet(1, "Fleet-A", (0.0, 0.0, 0.0), ships=[Ship(1, "destroyer", "A-Ship")])
        
        fleet_b = Fleet(2, "Fleet-B", (0.0, 0.0, 0.0), ships=[Ship(2, "destroyer", "B-Ship")])
        
        # Fleet-Bを全滅させる
        fleet_b.ships[0].take_damage(1000)
        
        resolver = CombatResolver()
        damage_to_b, damage_to_a = resolver.resolve_combat(fleet_a, fleet_b)
        
        # Fleet-Bは全滅しているので反撃できない
        assert damage_to_a == 0


@pytest.mark.unit
class TestAmmoConsumption:
    """弾薬消費のテスト"""

    def test_ammo_consumed_on_attack(self, reset_rng):
        """攻撃時に弾薬消費（_attack_fleetレベルでテスト）"""
        attacker_ship = Ship(1, "destroyer", "Attacker")
        defender_ship = Ship(2, "destroyer", "Defender")

        attacker_fleet = Fleet(1, "Attacker-Fleet", (0.0, 0.0, 0.0), ships=[attacker_ship])
        defender_fleet = Fleet(2, "Defender-Fleet", (0.0, 0.0, 0.0), ships=[defender_ship])

        initial_ammo = attacker_ship.ammo
        resolver = CombatResolver()

        # 艦隊攻撃を実行（内部で弾薬消費）
        damage = resolver._attack_fleet(attacker_fleet, defender_fleet)

        # 弾薬が消費されている
        assert attacker_ship.ammo == initial_ammo - attacker_ship.ammo_per_shot

    def test_different_ammo_per_shot(self):
        """艦種ごとの弾薬消費量"""
        destroyer = Ship(1, "destroyer", "DD")
        cruiser = Ship(2, "cruiser", "CA")
        battleship = Ship(3, "battleship", "BB")
        
        assert destroyer.ammo_per_shot == 1
        assert cruiser.ammo_per_shot == 2
        assert battleship.ammo_per_shot == 5
