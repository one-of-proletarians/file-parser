use serde::Serialize;

use std::{fs, path::Path};

#[derive(Serialize)]
pub struct Field {
    original: Original,
    translate: Translate,
    tags: Vec<String>,
}

#[derive(Serialize)]
struct Original {
    language: String,
    text: String,
}

#[derive(Serialize)]
struct Translate {
    language: String,
    text: String,
}

pub fn to_json(vector: &Vec<Field>) -> String {
    return serde_json::to_string(&vector).expect("failed to serialize to json");
}

/// Парсинг файла в формате .txt
///
/// Возвращает Vec<Field>, которые могут быть преобразованы в json
pub fn parse(
    file_path: &Path,
    original_language: &str,
    translate_language: &str,
) -> Box<Vec<Field>> {
    // Создание пустого Vec<Field> и пустого Vec<String> для хранения тегов
    let mut fields: Vec<Field> = Vec::new();
    let mut tags: Vec<String> = Vec::new();

    // Начальная строка для парсинга, по умолчанию 0 (первая строка файла)
    let mut start_line = 0;

    // Чтение файла в виде строки
    let content = fs::read_to_string(file_path).expect("failed to read file to string");

    // Создание Vec<&str> из строк файла
    let lines = content
        .split("\n") // Разбиение строки на отдельные строки
        .map(|x| x.trim()) // Удаление пробелов в начале и конце строки
        .filter(|x| !x.is_empty()) // Фильтрация пустых строк
        .collect::<Vec<&str>>(); // Преобразование в вектор строк

    // Получение разделителя между original и translate
    let separator = match get_separator(lines[0]) {
        // Если строка начинается с "@sep", то возвращаем разделитель
        // в виде строки, иначе возвращаем DEFAULT_SEPARATOR из dotenv
        Err(_) => dotenv!("DEFAULT_SEPARATOR").to_string(),
        Ok(x) => {
            // Если разделитель найден, то начинаем парсинг файла с первой строки
            // иначе с нулевой
            start_line = 1;
            x
        }
    };

    // Проход по всем строкам файла
    for index in start_line..lines.len() {
        let line = lines[index];

        // Если строка начинается с "#", то это начало области видимости тега
        if line.starts_with("#") {
            let tag = line.replace("#", "");

            // Если строка начинается с "##", то это конец области видимости тега
            if line.starts_with("##") {
                // Если тег уже есть в Vec<String>, то удаляем его
                match tags.iter().position(|x| x == &tag) {
                    Some(i) => tags.remove(i),
                    // Если тега нет, то ничего не делаем
                    None => "".to_string(),
                };
            } else {
                // добавляем тег в Vec<String>
                tags.push(tag);
            }
        } else {
            // Если строка не начинается с "#", то это original и translate
            let (original, translate) = match line.split_once(separator.as_str()) {
                // Если разделитель найден, то разбиваем строку на original и translate
                Some(x) => x,
                // Если разделитель не найден, то original = строка, translate = пустая строка
                None => (line, ""),
            };

            // Создание нового поля и добавление его в Vec<Field> fields
            fields.push(Field {
                original: Original {
                    language: String::from(original_language),
                    text: String::from(original.trim()),
                },
                translate: Translate {
                    language: String::from(translate_language),
                    text: String::from(translate.trim()),
                },
                tags: tags.clone(),
            });
        }
    }

    // Возвращаем Vec<Field> в Box<Vec<Field>>
    return Box::new(fields);
}

/// Получение разделителя между original и translate в файле
///
/// Если строка начинается с "@sep", то возвращаем разделитель
/// в виде строки, иначе возвращаем Err
fn get_separator(line: &str) -> Result<String, ()> {
    if line.starts_with("@sep") {
        Ok(line[5..].to_string())
    } else {
        Err(())
    }
}
