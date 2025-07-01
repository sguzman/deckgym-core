
import json
import re

def audit_basic_pokemon():
    with open('database.json') as f:
        database = json.load(f)

    with open('src/actions/apply_attack_action.rs') as f:
        attack_impl = f.read()

    implemented_attacks = set(re.findall(r'AttackId::(\w+)', attack_impl))

    unimplemented_attacks = []

    for card in database:
        if 'Pokemon' in card:
            pokemon = card['Pokemon']
            # Filter for Basic Pokemon (stage 0)
            if pokemon.get('stage') == 0:
                card_name = pokemon['name']
                if pokemon.get('attacks'):
                    for i, attack in enumerate(pokemon['attacks']):
                        # Normalize the attack ID by removing non-alphanumeric characters
                        normalized_attack_id = re.sub(r'[^a-zA-Z0-9]', '', attack['title'])

                        # Construct the enum variant name
                        attack_id_enum = f'{pokemon["id"].replace(" ", "")}{normalized_attack_id}'

                        if attack_id_enum not in implemented_attacks:
                            unimplemented_attacks.append(f'{card_name}: {attack["title"]}')

    print("--- Unimplemented Attacks for Basic Pokémon ---")
    for attack in sorted(list(set(unimplemented_attacks))):
        print(attack)

if __name__ == "__main__":
    audit_basic_pokemon()
