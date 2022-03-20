use reqwest::Url;

use crate::AppError;

pub async fn get_songs_as_rss(url: url::Url) -> Result<Vec<rss::Item>, AppError> {
  // 迭代引发
  let (mut songs, next) = get_next_songs(url.clone(), None).await?;
  // 迭代增长
  if next.is_some() {
    let mut new_next = next.unwrap();
    loop {
      let (songs_next, next_res) = get_next_songs(url.clone(), Some(new_next)).await?;
      songs.extend(songs_next);
      if let Some(new_next_res) = next_res {
        new_next = new_next_res;
      } else {
        break;
      }
    }
  }
  Ok(songs)
}
async fn get_next_songs(
  mut url: Url,
  next: Option<String>,
) -> Result<(Vec<rss::Item>, Option<String>), AppError> {
  if !url.path().ends_with("/api") && !url.path().ends_with("/api/") {
    let mut dir_path = url.path().to_string();
    if !dir_path.ends_with("/") {
      dir_path.push_str("/");
    }
    url.set_path("api/");
    url.query_pairs_mut().append_pair("path", &dir_path);
  }
  let url_clone = url.clone();
  if let Some(next) = next {
    url.query_pairs_mut().append_pair("next", &next);
  }
  let res: serde_json::Value = reqwest::get(url).await?.json().await?;
  let next_data = res
    .get("next")
    .and_then(|v| v.as_str().and_then(|v| Some(v.to_string())));
  let songs_data = res
    .get("folder")
    .unwrap()
    .get("value")
    .and_then(|v| v.as_array())
    .unwrap();
  let mut songs = Vec::new();
  for song_data in songs_data {
    if song_data.get("folder") != None {
      continue;
    }
    let name = song_data
      .get("name")
      .and_then(|v| v.as_str())
      .unwrap()
      .to_string();
    let date = chrono::DateTime::parse_from_rfc3339(
      song_data
        .get("lastModifiedDateTime")
        .and_then(|v| v.as_str())
        .unwrap(),
    )
    .unwrap();

    let rawfile_url = {
      let mut url = url_clone.clone();
      url.set_path(&format!("{}raw/", url.path()));
      url.set_query(Some(&format!("{}{}", url.query().unwrap(), name)));
      url.to_string()
    };
    let item = rss::ItemBuilder::default()
      .title(name.to_string())
      .enclosure(
        rss::EnclosureBuilder::default()
          .url(rawfile_url.clone())
          .length(
            song_data
              .get("size")
              .and_then(|v| v.as_u64())
              .unwrap()
              .to_string(),
          )
          .mime_type({
            if name.ends_with(".flac") || name.ends_with(".FLAC") {
              "audio/flac".to_string()
            } else {
              song_data
                .get("file")
                .unwrap()
                .get("mimeType")
                .and_then(|v| v.as_str())
                .unwrap()
                .to_string()
            }
          })
          .build(),
      )
      .guid(rss::Guid {
        value: song_data
          .get("id")
          .and_then(|v| v.as_str())
          .unwrap()
          .to_string(),
        permalink: false,
      })
      .pub_date(date.to_rfc2822())
      .build();
    songs.push(item);
  }
  Ok((songs, next_data))
}
