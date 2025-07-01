import json
import re

def audit_project():
    with open('database.json') as f:
        database = json.load(f)

    with open('src/actions/apply_attack_action.rs') as f:
        attack_impl = f.read()

    with open('src/actions/apply_abilities_action.rs') as f:
        ability_impl = f.read()

    implemented_attacks = set(re.findall(r'AttackId::(\w+)', attack_impl))
    implemented_abilities = set(re.findall(r'AbilityId::(\w+)', ability_impl))

    unimplemented_attacks = []
    unimplemented_abilities = []

    for card in database:
        if 'Pokemon' in card:
            pokemon = card['Pokemon']
            card_name = pokemon['name']
            if pokemon.get('attacks'):
                for i, attack in enumerate(pokemon['attacks']):
                    attack_id = re.sub(r'[^a-zA-Z0-9]', '', f'{pokemon["id"]}{attack["title"]}')
                    attack_id_enum = f'{pokemon["id"].replace(" ", "")}{attack_id}'
                    
                    # Normalize the attack ID by removing non-alphanumeric characters
                    normalized_attack_id = re.sub(r'[^a-zA-Z0-9]', '', attack['title'])
                    
                    # Construct the enum variant name
                    attack_id_enum = f'{pokemon["id"].replace(" ", "")}{normalized_attack_id}'

                    if attack_id_enum not in implemented_attacks:
                        unimplemented_attacks.append(f'{card_name}: {attack["title"]}')

            if pokemon.get('ability'):
                ability = pokemon['ability']
                ability_id = re.sub(r'[^a-zA-Z0-9]', '', pokemon['name'])
                ability_id_enum = f'{pokemon["id"].replace(" ", "")}{ability_id}'
                if ability_id_enum not in implemented_abilities:
                    unimplemented_abilities.append(f'{card_name}: {ability["title"]}')

    print("--- Unimplemented Attacks ---")
    for attack in sorted(list(set(unimplemented_attacks))):
        print(attack)

    print("\n--- Unimplemented Abilities ---")
    for ability in sorted(list(set(unimplemented_abilities))):
        print(ability)

if __name__ == "__main__":
    audit_project()
