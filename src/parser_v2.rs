use regex::Regex;
use serde::Serialize;

use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::Path,
};

/// Структура, описывающая результат парсинга файла с помощью парсера `v2`.
///
/// Структура содержит информацию о языках (`languages`), полях (`fields`),
/// и ошибках (`errors`), которые были найдены во время парсинга.
#[derive(Serialize)]
pub struct Response {
    languages: Languages,
    fields: Vec<Field>,
    errors: Vec<ErrorLine>,
}

/// Структура, описывающая отдельный текст для перевода.
///
/// Структура содержит оригинальный текст (`original`) и его перевод (`translate`).
#[derive(Serialize, Clone)]
struct Text {
    original: String,
    translate: String,
}

/// Структура, описывающая поле в файле.
///
/// Структура содержит набор тегов (`tags`), с помощью которых
/// поле можно идентифицировать, и вектор текстов для перевода (`content`).
#[derive(Serialize)]
struct Field {
    tags: HashSet<String>,
    content: Vec<Text>,
}

/// Структура, описывающая языки, используемые в файле для перевода.
///
/// Структура содержит идентификатор языка оригинала (`original`) и идентификатор языка перевода (`translate`).
#[derive(Serialize)]
struct Languages {
    original: String,
    translate: String,
}

/// Структура, описывающая строку с ошибкой при парсинге файла.
///
/// Структура содержит номер строки (`line`), в которой была найдена ошибка,
/// и вектор индексов столбцов (`columns`), в которых были найдены ошибки,
/// а также саму строку с ошибкой (`string`).
#[derive(Serialize)]
struct ErrorLine {
    line: i32,
    columns: Vec<usize>,
    string: String,
}

/// Описывает функцию, которая парсит файл и создает объект-ответ.
///
/// Параметр `path_to_file: &`[`Path`] - путь до файла, который нужно парсить.
///
/// Функция возвращает `Result<Box<Response>, ()>`, где [`Ok`] - успешно
/// пропарсенный объект-ответ, а [`Err`] - ошибка при чтении или парсинге файла.
pub fn parse(path_to_file: &Path) -> Result<Box<Response>, ()> {
    let file = match File::open(path_to_file) {
        Ok(file) => file,
        Err(_) => return Err(()),
    };

    let mut reader = BufReader::new(&file);

    let mut response = Response {
        fields: Default::default(),
        errors: Default::default(),
        languages: Languages {
            original: "ru".to_string(),
            translate: "de".to_string(),
        },
    };

    let mut content: Vec<Text> = Default::default();
    let mut tags: HashSet<String> = Default::default();
    let sep = get_separator(&mut reader);

    let mut string: String;
    let mut num_line: i32 = 0;

    let tags_reg = Regex::new(r"(^#{1,2}\w+)|(^@{1,2}tags)").unwrap();
    let error_reg = Regex::new("[<>:\"/\\|*]+").unwrap();
    let remove_tags_reg = Regex::new(r"^(#{2})|(@{2}tags\s)").unwrap();

    for line in reader.lines() {
        num_line += 1;

        string = match line {
            Ok(x) => x.trim().to_string(),
            Err(_) => "".to_string(),
        };

        if skip_line_else(&string) {
            continue;
        }

        if error_reg.is_match(&string) {
            let mut error = ErrorLine {
                line: num_line,
                columns: Default::default(),
                string: string.to_string(),
            };

            for column in error_reg.find_iter(&string) {
                error.columns.push(column.start());
            }

            response.errors.push(error);

            continue;
        }

        if tags_reg.is_match(string.as_str()) {
            let parsed_tags = parse_tags(&string);

            update_response(&mut response, &mut content, &mut tags);

            if remove_tags_reg.is_match(&string) {
                substract_tags(&mut tags, &parsed_tags);
            } else {
                extend_tags(&mut tags, &parsed_tags);
            }
        } else {
            let (original, translate) = match string.split_once(sep.as_str()) {
                Some(x) => x,
                None => (string.as_str(), ""),
            };

            content.push(Text {
                original: String::from(original.trim()),
                translate: String::from(translate.trim()),
            });
        }
    }

    update_response(&mut response, &mut content, &mut tags);

    return Ok(Box::new(response));
}

/// Определяет, пустая ли строка или начинается ли она с комментария
/// (строка начинается с "//").
fn skip_line_else(string: &String) -> bool {
    let reg = Regex::new(r"^//|@sep").unwrap();
    return reg.is_match(string) || string.is_empty();
}

/// Описывает функцию, которая добавляет в объект-ответ новый элемент [`Field`], если в нём нет такого же набора тэгов.
/// Если же есть, то добавляет к нему содержимое из переданного вектора [`Field::content`].
/// Если вектор не пуст, то очищает его после добавления.
fn update_response(response: &mut Response, content: &mut Vec<Text>, tags: &mut HashSet<String>) {
    if !content.is_empty() {
        for field in response.fields.iter_mut() {
            if *tags == field.tags {
                field.content.append(content);
                return;
            }
        }

        response.fields.push(Field {
            tags: tags.clone(),
            content: content.clone(),
        });

        content.clear();
    }
}

/// Вычитает из набора тэгов набор тэгов, которые должны быть вычеркнуты
fn substract_tags(target_tags: &mut HashSet<String>, tags_to_substract: &Box<HashSet<String>>) {
    for tag in tags_to_substract.iter() {
        target_tags.remove(tag);
    }
}

/// Добавляет в набор тэгов набор тэгов, которые должны быть добавлены
fn extend_tags(target_tags: &mut HashSet<String>, additional_tags: &Box<HashSet<String>>) {
    for tag in additional_tags.iter() {
        target_tags.insert(tag.clone());
    }
}

/// Определяет набор тэгов из строки. Если строка начинается с символа @, то разбивает
/// остаток строки на набор тэгов, разделенных запятыми, и возвращает их в виде [`HashSet`].
/// Если строка начинается с символа #, то возвращает [`HashSet`], содержащий одну строку, без символа # в начале.
///
fn parse_tags(string: &String) -> Box<HashSet<String>> {
    let mut tags: HashSet<String> = Default::default();
    if string.starts_with("@") {
        let raw = string.replace("@", "")[4..].to_string();
        let collect: HashSet<&str> = raw.split(",").map(|x| x.trim()).collect();

        for tag in collect {
            tags.insert(tag.to_string());
        }
    } else if string.starts_with("#") {
        let tag = string.replace("#", "").trim().to_string();
        tags.insert(tag);
    }

    return Box::new(tags);
}

/// Определяет разделитель, который будет использоваться при парсинге файла.
///
/// Если в начале файла есть строка `"@sep <разделитель>"`, то будет использован указанный разделитель.
/// В противном случае будет использован разделитель, заданный в настройках по умолчанию.
///
fn get_separator(reader: &mut BufReader<&File>) -> String {
    let mut separator = dotenv!("DEFAULT_SEPARATOR").to_string();

    for line in reader.lines() {
        let string = line.unwrap().trim().to_string();

        if string.starts_with("@sep ") {
            separator = string.replace("@sep ", "").trim().to_string();
            break;
        } else if !string.is_empty() {
            break;
        }
    }

    reader.seek(SeekFrom::Start(0)).unwrap();

    return separator;
}
