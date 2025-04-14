INSERT INTO {{ entity_plural_name }} (
    {% for field in changeset_struct_fields -%}
    {{ field.name }}{% unless forloop.last %}, {% endunless %}
    {%- endfor %}
) VALUES (
    {% for field in changeset_struct_fields -%}
    '{{ field.name }}'{% unless forloop.last %}, {% endunless %}
    {%- endfor %}
); 