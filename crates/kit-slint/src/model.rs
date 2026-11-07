//! Helfer zur Erstellung und Umwandlung von Slint-Modellen (`ModelRc`).

use slint::{ModelRc, SharedString, VecModel};

/// Trait zur bequemen Umwandlung in ein `ModelRc<T>`.
///
/// Erspart das wiederkehrende `ModelRc::new(VecModel::from(...))`.
///
/// # Beispiel
/// ```rust,ignore
/// use kit_slint::ToSlintModel;
///
/// let items = vec!["A", "B", "C"];
/// ui.set_my_list(items.to_slint_model());
/// ```
pub trait ToSlintModel<T: 'static> {
    fn to_slint_model(self) -> ModelRc<T>;
}

impl<T: 'static + Clone> ToSlintModel<T> for Vec<T> {
    fn to_slint_model(self) -> ModelRc<T> {
        ModelRc::new(VecModel::from(self))
    }
}

impl<T: 'static + Clone, const N: usize> ToSlintModel<T> for [T; N] {
    fn to_slint_model(self) -> ModelRc<T> {
        ModelRc::new(VecModel::from(self.to_vec()))
    }
}

/// Iterator-Extension zur direkten Umwandlung eines Iterators in ein `ModelRc<T>`.
///
/// # Beispiel
/// ```rust,ignore
/// use kit_slint::CollectSlintModel;
///
/// let model = (0..10).map(|i| i * 2).collect_slint();
/// ui.set_numbers(model);
/// ```
pub trait CollectSlintModel<T: 'static + Clone>: Iterator<Item = T> + Sized {
    fn collect_slint(self) -> ModelRc<T> {
        let v: Vec<T> = self.collect();
        ModelRc::new(VecModel::from(v))
    }
}

impl<I, T> CollectSlintModel<T> for I
where
    I: Iterator<Item = T>,
    T: 'static + Clone,
{
}

/// Trait zur einfachen Konvertierung von String-Iteratoren in `ModelRc<SharedString>`.
pub trait CollectSlintStrings: Iterator + Sized {
    fn collect_slint_strings<S: AsRef<str>>(self) -> ModelRc<SharedString>
    where
        Self: Iterator<Item = S>,
    {
        let v: Vec<SharedString> = self.map(|s| SharedString::from(s.as_ref())).collect();
        ModelRc::new(VecModel::from(v))
    }
}

impl<I, S> CollectSlintStrings for I
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
}

/// Makro zum deklarativen Erstellen eines `ModelRc` ähnlich wie `vec![...]`.
///
/// # Beispiel
/// ```rust,ignore
/// use kit_slint::slint_model;
///
/// let model = slint_model![1, 2, 3, 4, 5];
/// ```
#[macro_export]
macro_rules! slint_model {
    () => {
        $crate::slint::ModelRc::new($crate::slint::VecModel::default())
    };
    ($($elem:expr),+ $(,)?) => {
        $crate::slint::ModelRc::new($crate::slint::VecModel::from(vec![$($elem),+]))
    };
}

/// Makro zum deklarativen Erstellen eines `ModelRc<SharedString>` aus String-Literalen.
///
/// # Beispiel
/// ```rust,ignore
/// use kit_slint::slint_strings;
///
/// let model = slint_strings!["Tab 1", "Tab 2", "Tab 3"];
/// ```
#[macro_export]
macro_rules! slint_strings {
    () => {
        $crate::slint::ModelRc::new($crate::slint::VecModel::default())
    };
    ($($elem:expr),+ $(,)?) => {
        $crate::slint::ModelRc::new($crate::slint::VecModel::from(vec![
            $($crate::slint::SharedString::from($elem)),+
        ]))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::Model;

    #[test]
    fn test_to_slint_model_vec() {
        let v = vec![10, 20, 30];
        let model = v.to_slint_model();
        assert_eq!(model.row_count(), 3);
        assert_eq!(model.row_data(0), Some(10));
        assert_eq!(model.row_data(1), Some(20));
        assert_eq!(model.row_data(2), Some(30));
    }

    #[test]
    fn test_collect_slint() {
        let model = (1..=4).map(|x| x * 10).collect_slint();
        assert_eq!(model.row_count(), 4);
        assert_eq!(model.row_data(0), Some(10));
        assert_eq!(model.row_data(3), Some(40));
    }

    #[test]
    fn test_collect_slint_strings() {
        let strings = ["apple", "banana", "cherry"];
        let model = strings.iter().collect_slint_strings();
        assert_eq!(model.row_count(), 3);
        assert_eq!(model.row_data(1), Some(SharedString::from("banana")));
    }

    #[test]
    fn test_slint_macros() {
        let m1 = slint_model![100, 200];
        assert_eq!(m1.row_count(), 2);

        let m2 = slint_strings!["foo", "bar", "baz"];
        assert_eq!(m2.row_count(), 3);
        assert_eq!(m2.row_data(0), Some(SharedString::from("foo")));
    }
}
